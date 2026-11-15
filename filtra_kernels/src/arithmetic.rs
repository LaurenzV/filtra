use fearless_simd::{Level, Simd, dispatch, f32x8, prelude::*};

use crate::pixmap::segments;
use crate::simd::{for_each_chunk, pack, unpack};
use crate::{Source, Target};

/// Combines `src` and `backdrop` with `k1·i1·i2 + k2·i1 + k3·i2 + k4` per premultiplied channel.
pub fn arithmetic(src: Source<'_>, backdrop: Source<'_>, k: [f32; 4], dst: Target<'_>) {
    dispatch!(Level::new(), simd => arithmetic_inner(simd, src, backdrop, k, dst));
}

#[inline(always)]
fn arithmetic_inner<S: Simd>(
    simd: S,
    src: Source<'_>,
    backdrop: Source<'_>,
    k: [f32; 4],
    mut dst: Target<'_>,
) {
    let rect = dst.rect();
    let k1 = f32x8::splat(simd, k[0] / 255.0);
    let k2 = f32x8::splat(simd, k[1]);
    let k3 = f32x8::splat(simd, k[2]);
    let k4 = f32x8::splat(simd, k[3] * 255.0);

    let formula = {
        #[inline(always)]
        |i1: f32x8<S>, i2: f32x8<S>| (k1 * i1).mul_add(i2, k2.mul_add(i1, k3.mul_add(i2, k4)))
    };

    for (y, row) in dst.rows_mut() {
        for (x0, x1) in segments(rect.x0, rect.x1, [src.rect(), backdrop.rect()].into_iter()) {
            let output = &mut row[(x0 - rect.x0) as usize * 4..(x1 - rect.x0) as usize * 4];
            let inputs = [src.row(y, x0, x1), backdrop.row(y, x0, x1)];

            for_each_chunk(inputs, output, {
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
    }
}
