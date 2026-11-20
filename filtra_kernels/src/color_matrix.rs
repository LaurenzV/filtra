use fearless_simd::{Level, Simd, dispatch, f32x8, prelude::*};

use crate::simd::{for_each_pixel_chunk, pack, premultiply, unpack, unpremultiply};
use crate::{Source, Target};

/// `feColorMatrix`, with every `type` given as its matrix.
pub fn color_matrix(src: Source<'_>, dst: Target<'_>, matrix: &[f32; 20]) {
    dispatch!(Level::new(), simd => color_matrix_inner(simd, src, dst, matrix));
}

#[inline(always)]
fn color_matrix_inner<S: Simd>(simd: S, src: Source<'_>, dst: Target<'_>, matrix: &[f32; 20]) {
    let m: [f32x8<S>; 20] = core::array::from_fn({
        #[inline(always)]
        |i| f32x8::splat(simd, matrix[i])
    });

    let transform = {
        #[inline(always)]
        |channel: usize, [r, g, b, a]: [f32x8<S>; 4]| {
            let m = &m[channel * 5..channel * 5 + 5];
            m[0].mul_add(r, m[1].mul_add(g, m[2].mul_add(b, m[3].mul_add(a, m[4]))))
        }
    };

    for_each_pixel_chunk([src], dst, {
        #[inline(always)]
        |[source]: [&[u8; 32]; 1], output: &mut [u8; 32]| {
            let color = unpremultiply(unpack(simd, source));
            let transformed = [
                transform(0, color),
                transform(1, color),
                transform(2, color),
                transform(3, color),
            ];

            pack(premultiply(transformed), output);
        }
    });
}
