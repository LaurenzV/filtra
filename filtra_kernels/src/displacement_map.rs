use fearless_simd::{Level, Simd, dispatch, f32x8, i32x8, prelude::*};

use crate::pixmap::segments;
use crate::simd::{for_each_chunk, unpack, unpremultiply};
use crate::{Source, Target};

/// A color or alpha channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorChannel {
    /// Red.
    R,
    /// Green.
    G,
    /// Blue.
    B,
    /// Alpha.
    A,
}

/// Copies to each pixel the pixel of `src` that contains its center moved by `scale` times the
/// unpremultiplied `channels` of `map` minus one half.
pub fn displacement_map(
    src: Source<'_>,
    map: Source<'_>,
    dst: Target<'_>,
    scale: [f32; 2],
    channels: [ColorChannel; 2],
) {
    dispatch!(Level::new(), simd => displacement_map_inner(simd, src, map, dst, scale, channels));
}

#[inline(always)]
fn displacement_map_inner<S: Simd>(
    simd: S,
    src: Source<'_>,
    map: Source<'_>,
    mut dst: Target<'_>,
    scale: [f32; 2],
    channels: [ColorChannel; 2],
) {
    let rect = dst.rect();
    // An infinite scale would make a channel of one half displace by NaN.
    let [scale_x, scale_y] =
        scale.map(|scale| f32x8::splat(simd, scale.clamp(-f32::MAX, f32::MAX)));
    let [channel_x, channel_y] = channels.map(|channel| channel as usize);
    let lanes = f32x8::simd_from(simd, core::array::from_fn(|i| i as f32 + 0.5));

    for (y, row) in dst.rows_mut() {
        let center_y = f32x8::splat(simd, y as f32 + 0.5);

        for (x0, x1) in segments(rect.x0, rect.x1, [map.rect()].into_iter()) {
            let output = &mut row[(x0 - rect.x0) as usize * 4..(x1 - rect.x0) as usize * 4];
            let mut x = x0;

            for_each_chunk([map.row(y, x0, x1)], output, {
                #[inline(always)]
                |[map]: [&[u8; 32]; 1], output: &mut [u8; 32]| {
                    let channels = unpremultiply(unpack(simd, map));
                    let center_x = lanes + x as f32;
                    let source = {
                        #[inline(always)]
                        |scale: f32x8<S>, channel: usize, center: f32x8<S>| {
                            let moved = scale.mul_add(channels[channel] - 0.5, center);
                            moved.floor().to_int::<i32x8<S>>()
                        }
                    };
                    let source_x = source(scale_x, channel_x, center_x);
                    let source_y = source(scale_y, channel_y, center_y);
                    let pixels = output.as_chunks_mut::<4>().0.iter_mut();

                    for (pixel, (&x, &y)) in pixels.zip(source_x.iter().zip(source_y.iter())) {
                        *pixel = *src.pixel(x, y).unwrap_or(&[0; 4]);
                    }

                    x += 8;
                }
            });
        }
    }
}
