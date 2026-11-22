use fearless_simd::{Simd, f32x8, i32x8, prelude::*};
use filtra_types::ColorChannel;

use crate::dispatch::dispatch;
use crate::pixmap::for_each_segment;
use crate::simd::{for_each_chunk, unpack, unpremultiply};
use crate::{Source, Target};

/// `feDisplacementMap`, with `scale` in pixels.
pub fn displacement_map(
    src: Source<'_>,
    map: Source<'_>,
    dst: Target<'_>,
    scale: [f32; 2],
    channels: [ColorChannel; 2],
) {
    dispatch!(simd => displacement_map_inner(simd, src, map, dst, scale, channels));
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
    // An infinite scale would make a channel of one half displace by NaN.
    let [scale_x, scale_y] =
        scale.map(|scale| f32x8::splat(simd, scale.clamp(-f32::MAX, f32::MAX)));
    let [channel_x, channel_y] = channels.map(|channel| channel as usize);
    let lanes = f32x8::simd_from(simd, core::array::from_fn(|i| i as f32));

    for_each_segment(&mut dst, [map.rect()].into_iter(), {
        #[inline(always)]
        |y, x0, x1, output| {
            let pixel_y = f32x8::splat(simd, y as f32);

            for_each_chunk([map.row(y, x0, x1)], output, {
                #[inline(always)]
                |i, [map]: [&[u8; 32]; 1], output: &mut [u8; 32]| {
                    let channels = unpremultiply(unpack(simd, map));
                    let pixel_x = lanes + (x0 + 8 * i as i32) as f32;
                    // Rounding the displacement half away from zero treats both directions alike,
                    // so that flipping an axis mirrors the result.
                    let source = {
                        #[inline(always)]
                        |scale: f32x8<S>, channel: usize, pixel: f32x8<S>| {
                            let displacement = scale * (channels[channel] - 0.5);
                            (displacement.round() + pixel).to_int::<i32x8<S>>()
                        }
                    };
                    let source_x = source(scale_x, channel_x, pixel_x);
                    let source_y = source(scale_y, channel_y, pixel_y);
                    let pixels = output.as_chunks_mut::<4>().0.iter_mut();

                    for (pixel, (&x, &y)) in pixels.zip(source_x.iter().zip(source_y.iter())) {
                        *pixel = *src.pixel(x, y).unwrap_or(&[0; 4]);
                    }
                }
            });
        }
    });
}
