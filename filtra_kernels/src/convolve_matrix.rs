use fearless_simd::{Level, Simd, dispatch, f32x8, prelude::*};

use crate::simd::{for_each_chunk, pack, premultiply, unpack, unpremultiply};
use crate::{IntRect, Source, Target};

/// How [`convolve_matrix`] extends its input beyond the edges of the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeMode {
    /// Repeats the pixels at the edges.
    Duplicate,
    /// Takes the pixels from the opposite edge.
    Wrap,
    /// Takes the pixels mirrored across the edges.
    Mirror,
    /// Extends with transparent black.
    None,
}

/// The parameters of [`convolve_matrix`].
#[derive(Debug, Clone, Copy)]
pub struct Convolution<'a> {
    /// The number of columns and rows of `weights`.
    pub size: [u32; 2],
    /// The weights in row-major order, each multiplying the pixel it covers.
    pub weights: &'a [f32],
    /// The column and row of the weight that covers the pixel being computed.
    pub target: [u32; 2],
    /// The value added to alpha, and times alpha to each color.
    pub bias: f32,
    /// How the input is extended beyond the edges of the target.
    pub edge_mode: EdgeMode,
    /// Whether only the unpremultiplied colors are convolved, keeping alpha.
    pub preserve_alpha: bool,
}

/// Sets each pixel to the sum of the pixels of `src` around it, weighted by `convolution`.
pub fn convolve_matrix(src: Source<'_>, dst: Target<'_>, convolution: &Convolution<'_>) {
    let [columns, rows] = convolution.size;
    let [target_x, target_y] = convolution.target;

    assert_eq!(
        Some(convolution.weights.len() as u64),
        u64::from(columns).checked_mul(u64::from(rows)),
        "the number of weights doesn't match the size"
    );
    assert!(
        target_x < columns && target_y < rows,
        "the target is outside the weights"
    );

    dispatch!(Level::new(), simd => convolve(simd, src, dst, convolution));
}

#[inline(always)]
fn convolve<S: Simd>(simd: S, src: Source<'_>, mut dst: Target<'_>, convolution: &Convolution<'_>) {
    let rect = dst.rect();

    if rect.is_empty() {
        return;
    }

    // The rows of the input that the weights cover, extended by the edge mode, are kept as
    // channel planes of `f32`: in a ring with one slot per row of weights, or with one slot per
    // row of the input (and one for rows that `none` leaves transparent) if the weights are
    // taller than it. Each chunk of 8 output pixels then sums its weighted neighbors in registers.
    let [columns, rows] = convolution.size.map(|n| n as usize);
    let height = rect.height() as usize;
    let input = ExtendedRows::new(rect, convolution);
    let every_row = rows > height;
    let slot_len = 4 * input.stride;
    let capacity = if every_row { height + 1 } else { rows };
    let mut cache = vec![0.0; capacity * slot_len];
    let mut held = vec![None; capacity];
    let preserve_alpha = convolution.preserve_alpha;
    let channels = if preserve_alpha { 3 } else { 4 };
    let bias = f32x8::splat(simd, convolution.bias);
    let mut slots = vec![0; rows];

    for (y, (_, output)) in dst.rows_mut().enumerate() {
        for (i, slot) in slots.iter_mut().enumerate() {
            let row = y + i;
            let input_row = extend(
                row as i64 - i64::from(convolution.target[1]),
                height,
                convolution.edge_mode,
            );
            let index = if every_row {
                input_row.unwrap_or(height)
            } else {
                row % rows
            };

            if held[index] != Some(input_row) {
                let planes = &mut cache[index * slot_len..][..slot_len];
                input.fill(simd, src, input_row, planes, preserve_alpha);
                held[index] = Some(input_row);
            }

            *slot = index * slot_len;
        }

        let center = slots[convolution.target[1] as usize] + 3 * input.stride + input.target_x;
        let mut x = 0;

        for_each_chunk::<0>([], output, {
            #[inline(always)]
            |[], output: &mut [u8; 32]| {
                let mut sums = [f32x8::splat(simd, 0.0); 4];

                for (&slot, weights) in slots.iter().zip(convolution.weights.chunks_exact(columns))
                {
                    // Slicing each plane once per row of weights leaves one bounds check per
                    // weight instead of one per load.
                    let windows: [&[f32]; 4] = core::array::from_fn(|channel| {
                        &cache[slot + channel * input.stride + x..][..columns + 7]
                    });

                    for (j, &weight) in weights.iter().enumerate() {
                        if weight == 0.0 {
                            continue;
                        }

                        let weight = f32x8::splat(simd, weight);

                        for (sum, window) in sums.iter_mut().zip(&windows).take(channels) {
                            *sum = f32x8::from_slice(simd, &window[j..j + 8]).mul_add(weight, *sum);
                        }
                    }
                }

                let [r, g, b, a] = sums;
                let pixels = if preserve_alpha {
                    let alpha = f32x8::from_slice(simd, &cache[center + x..][..8]);
                    let bias = bias * alpha;
                    premultiply([r + bias, g + bias, b + bias, alpha])
                } else {
                    let alpha = (a + bias).max(0.0).min(1.0);
                    let bias = bias * alpha;
                    let color = {
                        #[inline(always)]
                        |sum: f32x8<S>| (sum + bias).max(0.0).min(alpha) * 255.0
                    };
                    [color(r), color(g), color(b), alpha * 255.0]
                };

                pack(pixels, output);
                x += 8;
            }
        });
    }
}

struct ExtendedRows {
    rect: IntRect,
    width: usize,
    extended: usize,
    // Leaves room for reading a full chunk past the last pixel.
    stride: usize,
    target_x: usize,
    edge_mode: EdgeMode,
}

impl ExtendedRows {
    fn new(rect: IntRect, convolution: &Convolution<'_>) -> Self {
        let width = rect.width() as usize;
        let extended = width + convolution.size[0] as usize - 1;

        Self {
            rect,
            width,
            extended,
            stride: extended + 8,
            target_x: convolution.target[0] as usize,
            edge_mode: convolution.edge_mode,
        }
    }

    #[inline(always)]
    fn fill<S: Simd>(
        &self,
        simd: S,
        src: Source<'_>,
        row: Option<usize>,
        planes: &mut [f32],
        preserve_alpha: bool,
    ) {
        planes.fill(0.0);

        let Some(row) = row else {
            return;
        };

        let source = src.rect();
        let (x0, x1) = (self.rect.x0.max(source.x0), self.rect.x1.min(source.x1));

        if let Some(pixels) = src.row(self.rect.y0 + row as i32, x0, x1) {
            let start = self.target_x + (x0 - self.rect.x0) as usize;
            let (chunks, tail) = pixels.as_chunks::<32>();
            let mut padded = [0; 32];
            padded[..tail.len()].copy_from_slice(tail);
            let padded = (!tail.is_empty()).then_some(&padded);

            for (k, chunk) in chunks.iter().chain(padded).enumerate() {
                let channels = unpack(simd, chunk);
                let values = if preserve_alpha {
                    unpremultiply(channels)
                } else {
                    let [r, g, b, a] = channels;
                    let scale = 1.0 / 255.0;
                    [r * scale, g * scale, b * scale, a * scale]
                };

                for (channel, value) in values.into_iter().enumerate() {
                    value.store_slice(&mut planes[channel * self.stride + start + 8 * k..][..8]);
                }
            }
        }

        let border = (0..self.target_x).chain(self.target_x + self.width..self.extended);

        for column in border {
            let x = column as i64 - self.target_x as i64;
            let Some(x) = extend(x, self.width, self.edge_mode) else {
                continue;
            };

            for channel in 0..4 {
                let plane = &mut planes[channel * self.stride..][..self.stride];
                plane[column] = plane[self.target_x + x];
            }
        }
    }
}

fn extend(index: i64, len: usize, edge_mode: EdgeMode) -> Option<usize> {
    let len = len as i64;

    if (0..len).contains(&index) {
        return Some(index as usize);
    }

    let index = match edge_mode {
        EdgeMode::Duplicate => index.clamp(0, len - 1),
        EdgeMode::Wrap => index.rem_euclid(len),
        EdgeMode::Mirror => match index.rem_euclid(2 * len) {
            reflected if reflected < len => reflected,
            reflected => 2 * len - 1 - reflected,
        },
        EdgeMode::None => return None,
    };

    Some(index as usize)
}
