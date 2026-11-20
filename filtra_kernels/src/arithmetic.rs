use fearless_simd::{Simd, f32x8, prelude::*};

use crate::simd::{for_each_pixel_chunk, pack, unpack};
use crate::{Source, Target};

#[inline(always)]
pub(crate) fn arithmetic<S: Simd>(simd: S, inputs: [Source<'_>; 2], dst: Target<'_>, k: [f32; 4]) {
    let k1 = f32x8::splat(simd, k[0] / 255.0);
    let k2 = f32x8::splat(simd, k[1]);
    let k3 = f32x8::splat(simd, k[2]);
    let k4 = f32x8::splat(simd, k[3] * 255.0);

    let formula = {
        #[inline(always)]
        |i1: f32x8<S>, i2: f32x8<S>| (k1 * i1).mul_add(i2, k2.mul_add(i1, k3.mul_add(i2, k4)))
    };

    for_each_pixel_chunk(inputs, dst, {
        #[inline(always)]
        |[s, b]: [&[u8; 32]; 2], output: &mut [u8; 32]| {
            let [sr, sg, sb, sa] = unpack(simd, s);
            let [br, bg, bb, ba] = unpack(simd, b);
            let alpha = formula(sa, ba).max(0.0).min(255.0);
            let color = {
                #[inline(always)]
                |i1: f32x8<S>, i2: f32x8<S>| formula(i1, i2).max(0.0).min(alpha)
            };

            pack([color(sr, br), color(sg, bg), color(sb, bb), alpha], output);
        }
    });
}
