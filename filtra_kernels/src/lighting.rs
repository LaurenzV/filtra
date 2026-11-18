use fearless_simd::{Level, Simd, dispatch, f32x8, prelude::*};

use crate::simd::{for_each_chunk, pack, pow};
use crate::{IntRect, Source, Target};

/// Lighting of the alpha of an image as a height map (`feDiffuseLighting`, `feSpecularLighting`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lighting {
    /// The reflection model.
    pub reflection: Reflection,
    /// The height of the surface where alpha is 1, in user units.
    pub surface_scale: f32,
    /// The light source in user space.
    pub light: Light,
    /// The color of the light in the color space of the result.
    pub color: [f32; 3],
    /// The number of pixels per user unit along the x- and y-axis.
    pub scale: [f32; 2],
    /// The position of the user-space origin in pixels.
    pub origin: [f32; 2],
}

/// The reflection model of [`Lighting`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Reflection {
    /// The diffuse term of the Phong model, which is opaque.
    Diffuse {
        /// The diffuse constant `kd`.
        constant: f32,
    },
    /// The specular term of the Blinn-Phong model, whose alpha is its largest color.
    Specular {
        /// The specular constant `ks`.
        constant: f32,
        /// The specular exponent.
        exponent: f32,
    },
}

/// The light source of [`Lighting`], in user space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Light {
    /// An infinitely distant light.
    Distant {
        /// The unit vector pointing to the light.
        direction: [f32; 3],
    },
    /// A positional light.
    Point {
        /// The position.
        position: [f32; 3],
    },
    /// A positional light projecting a cone.
    Spot {
        /// The position.
        position: [f32; 3],
        /// The unit vector the light points along, or zero if it points at itself.
        direction: [f32; 3],
        /// The exponent controlling how focused the light is.
        exponent: f32,
        /// The cosine of the cone's half-angle, if the light has a cone.
        cone: Option<f32>,
    },
}

/// Lights the alpha of `src` as a height map, sampled at the top-left corner of each pixel.
pub fn lighting(src: Source<'_>, dst: Target<'_>, lighting: &Lighting) {
    dispatch!(Level::new(), simd => lighting_inner(simd, src, dst, lighting));
}

#[inline(always)]
fn lighting_inner<S: Simd>(simd: S, src: Source<'_>, mut dst: Target<'_>, lighting: &Lighting) {
    let rect = dst.rect();

    if rect.is_empty() {
        return;
    }

    let (width, height) = (rect.width() as usize, rect.height() as usize);
    // The rows above, at and below the current one, with a pixel of padding on each side and room
    // for reading a full chunk past the end.
    let mut rows: [Vec<f32>; 3] = core::array::from_fn(|_| vec![0.0; width + 10]);
    let mut slopes = [vec![0.0; width + 8], vec![0.0; width + 8]];

    alpha_row(src, rect, 0, &mut rows[1]);

    if height > 1 {
        alpha_row(src, rect, 1, &mut rows[2]);
    }

    for (y, (_, output)) in dst.rows_mut().enumerate() {
        if y > 0 {
            rows.rotate_left(1);

            if y + 1 < height {
                alpha_row(src, rect, y + 1, &mut rows[2]);
            }
        }

        let [above, center, below] = &rows;
        let above = (y > 0).then_some(above.as_slice());
        let below = (y + 1 < height).then_some(below.as_slice());
        gradient(simd, above, center, below, width, &mut slopes);
        shade(simd, lighting, rect, y, center, &slopes, output);
    }
}

// The horizontal and vertical slopes of the alpha, with the Sobel kernels of §9.10. At the edges,
// the padding extrapolates each row linearly, which turns the central differences into the
// one-sided ones of the spec, and missing rows drop out of the weights.
#[inline(always)]
fn gradient<S: Simd>(
    simd: S,
    above: Option<&[f32]>,
    center: &[f32],
    below: Option<&[f32]>,
    width: usize,
    [slope_x, slope_y]: &mut [Vec<f32>; 2],
) {
    let weight = {
        #[inline(always)]
        |row: Option<&[f32]>| if row.is_some() { 1.0 } else { 0.0 }
    };
    let (above_weight, below_weight) = (weight(above), weight(below));
    let rows_scale = 1.0 / (above_weight + 2.0 + below_weight);
    // A difference over one row instead of two counts twice.
    let span = above_weight + below_weight;
    let difference_scale = if span > 0.0 { 2.0 / span } else { 0.0 };
    let (above, below) = (above.unwrap_or(center), below.unwrap_or(center));
    let column_difference = {
        #[inline(always)]
        |x: usize| difference_scale * (below[x + 1] - above[x + 1])
    };

    for x in (0..width).step_by(8) {
        let load = {
            #[inline(always)]
            |row: &[f32], offset: usize| f32x8::from_slice(simd, &row[x + offset..][..8])
        };
        let horizontal = {
            #[inline(always)]
            |row: &[f32]| load(row, 2) - load(row, 0)
        };
        let vertical = {
            #[inline(always)]
            |offset: usize| (load(below, offset) - load(above, offset)) * difference_scale
        };

        let x_sum = horizontal(above) * above_weight
            + horizontal(center) * 2.0
            + horizontal(below) * below_weight;
        (x_sum * rows_scale).store_slice(&mut slope_x[x..][..8]);

        let y_sum = vertical(0) + vertical(1) * 2.0 + vertical(2);
        (y_sum * 0.25).store_slice(&mut slope_y[x..][..8]);
    }

    // The columns at the edges have a neighbor on one side only, which the vertical slopes weigh
    // by 2 and 1.
    if width == 1 {
        slope_y[0] = column_difference(0);
    } else {
        slope_y[0] = (2.0 * column_difference(0) + column_difference(1)) / 3.0;
        slope_y[width - 1] =
            (2.0 * column_difference(width - 1) + column_difference(width - 2)) / 3.0;
    }
}

#[inline(always)]
fn shade<S: Simd>(
    simd: S,
    lighting: &Lighting,
    rect: IntRect,
    y: usize,
    alpha: &[f32],
    [slope_x, slope_y]: &[Vec<f32>; 2],
    output: &mut [u8],
) {
    let [scale_x, scale_y] = lighting.scale;
    let surface_scale = lighting.surface_scale;
    // One pixel is `1 / scale` user units, which `dx` and `dy` of §9.10 are measured in.
    let normal_x = -surface_scale * scale_x;
    let normal_y = -surface_scale * scale_y;
    let lanes = f32x8::from_slice(simd, &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);
    let user_y = (rect.y0 as f32 + y as f32 - lighting.origin[1]) / scale_y;
    let [red, green, blue] = lighting.color;
    let mut x = 0;

    for_each_chunk::<0>([], output, {
        #[inline(always)]
        |[], output: &mut [u8; 32]| {
            let load = {
                #[inline(always)]
                |row: &[f32], offset: usize| f32x8::from_slice(simd, &row[x + offset..][..8])
            };
            let nx = load(slope_x, 0) * normal_x;
            let ny = load(slope_y, 0) * normal_y;
            let length = 1.0 / nx.mul_add(nx, ny.mul_add(ny, 1.0)).sqrt();
            let normal = [nx * length, ny * length, length];
            let user_x = (lanes + (rect.x0 as f32 + x as f32 - lighting.origin[0])) / scale_x;
            let point = [
                user_x,
                f32x8::splat(simd, user_y),
                load(alpha, 1) * surface_scale,
            ];

            let (to_light, intensity) = match lighting.light {
                Light::Distant {
                    direction: [to_x, to_y, to_z],
                } => {
                    let splat = {
                        #[inline(always)]
                        |v: f32| f32x8::splat(simd, v)
                    };
                    ([splat(to_x), splat(to_y), splat(to_z)], splat(1.0))
                }
                Light::Point { position } => {
                    let (to_light, _) = towards(position, point);
                    (to_light, f32x8::splat(simd, 1.0))
                }
                Light::Spot {
                    position,
                    direction,
                    exponent,
                    cone,
                } => {
                    let (to_light, inverse_distance) = towards(position, point);
                    let [sx, sy, sz] = direction;
                    let [lx, ly, lz] = to_light;
                    // The cosine of the angle between the light's axis and the surface point.
                    let cosine = -(lx * sx + ly * sy + lz * sz);
                    let lit = cosine
                        .simd_gt(0.0)
                        .select(pow(cosine, exponent), f32x8::splat(simd, 0.0));
                    let intensity = match cone {
                        // Anti-aliases the edge of the cone over one pixel, with the change of the
                        // cosine across a pixel from its derivatives along x and y, which are
                        // `(S + cosine · L) / distance`.
                        Some(limit) => {
                            let dx = cosine.mul_add(lx, sx) * inverse_distance * (1.0 / scale_x);
                            let dy = cosine.mul_add(ly, sy) * inverse_distance * (1.0 / scale_y);
                            let change = dx.mul_add(dx, dy * dy).sqrt();
                            let coverage = ((cosine - limit) / change + 0.5).max(0.0).min(1.0);
                            lit * coverage
                        }
                        None => lit,
                    };
                    (to_light, intensity)
                }
            };

            let [lx, ly, lz] = to_light;
            let [nx, ny, nz] = normal;
            let pixels = match lighting.reflection {
                Reflection::Diffuse { constant } => {
                    let factor = nx.mul_add(lx, ny.mul_add(ly, nz * lz)) * intensity * constant;
                    let channel = {
                        #[inline(always)]
                        |color: f32| (factor * color).max(0.0).min(1.0) * 255.0
                    };
                    [
                        channel(red),
                        channel(green),
                        channel(blue),
                        f32x8::splat(simd, 255.0),
                    ]
                }
                Reflection::Specular { constant, exponent } => {
                    // The halfway vector between the light and the eye, which is at (0, 0, 1).
                    let hz = lz + 1.0;
                    let length = 1.0 / lx.mul_add(lx, ly.mul_add(ly, hz * hz)).sqrt();
                    let cosine = nx.mul_add(lx, ny.mul_add(ly, nz * hz)) * length;
                    let factor = pow(cosine.max(0.0), exponent) * intensity * constant;
                    let channel = {
                        #[inline(always)]
                        |color: f32| (factor * color).max(0.0).min(1.0)
                    };
                    let [r, g, b] = [channel(red), channel(green), channel(blue)];
                    let a = r.max(g).max(b);
                    [r * 255.0, g * 255.0, b * 255.0, a * 255.0]
                }
            };

            pack(pixels, output);
            x += 8;
        }
    });
}

#[inline(always)]
fn towards<S: Simd>(position: [f32; 3], point: [f32x8<S>; 3]) -> ([f32x8<S>; 3], f32x8<S>) {
    let [px, py, pz] = point;
    let [dx, dy, dz] = [position[0] - px, position[1] - py, position[2] - pz];
    let distance = dx.mul_add(dx, dy.mul_add(dy, dz * dz)).sqrt();
    let inverse = 1.0 / distance;

    ([dx * inverse, dy * inverse, dz * inverse], inverse)
}

#[inline(always)]
fn alpha_row(src: Source<'_>, rect: IntRect, y: usize, row: &mut [f32]) {
    row.fill(0.0);

    let source = src.rect();
    let (x0, x1) = (rect.x0.max(source.x0), rect.x1.min(source.x1));

    if let Some(pixels) = src.row(rect.y0 + y as i32, x0, x1) {
        let start = 1 + (x0 - rect.x0) as usize;
        let pixels = pixels.as_chunks::<4>().0;
        let alphas = pixels
            .iter()
            .map(|pixel| f32::from(pixel[3]) * (1.0 / 255.0));

        for (value, alpha) in row[start..].iter_mut().zip(alphas) {
            *value = alpha;
        }
    }

    let width = rect.width() as usize;

    if width > 1 {
        row[0] = 2.0 * row[1] - row[2];
        row[width + 1] = 2.0 * row[width] - row[width - 1];
    } else {
        row[0] = row[1];
        row[2] = row[1];
    }
}
