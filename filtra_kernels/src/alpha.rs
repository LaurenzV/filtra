use fearless_simd::{Level, Simd, dispatch, prelude::*, u8x32, u32x8};

use crate::pixmap::segments;
use crate::simd::for_each_chunk;
use crate::{Source, Target};

/// Copies the alpha channel of `src` with black color.
pub fn alpha(src: Source<'_>, dst: Target<'_>) {
    dispatch!(Level::new(), simd => alpha_inner(simd, src, dst));
}

#[inline(always)]
fn alpha_inner<S: Simd>(simd: S, src: Source<'_>, mut dst: Target<'_>) {
    const ALPHA: u32 = u32::from_ne_bytes([0, 0, 0, 255]);

    let rect = dst.rect();

    for (y, row) in dst.rows_mut() {
        for (x0, x1) in segments(rect.x0, rect.x1, [src.rect()].into_iter()) {
            let output = &mut row[(x0 - rect.x0) as usize * 4..(x1 - rect.x0) as usize * 4];

            for_each_chunk([src.row(y, x0, x1)], output, {
                #[inline(always)]
                |[source]: [&[u8; 32]; 1], output: &mut [u8; 32]| {
                    let pixels: u32x8<S> = u8x32::from_slice(simd, source).bitcast();
                    (pixels & ALPHA).bitcast::<u8x32<S>>().store_slice(output);
                }
            });
        }
    }
}
