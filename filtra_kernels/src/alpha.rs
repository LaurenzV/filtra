use fearless_simd::{Level, Simd, dispatch, prelude::*, u8x32, u32x8};

use crate::simd::for_each_pixel_chunk;
use crate::{Source, Target};

/// The `SourceAlpha` input.
pub fn alpha(src: Source<'_>, dst: Target<'_>) {
    dispatch!(Level::new(), simd => alpha_inner(simd, src, dst));
}

#[inline(always)]
fn alpha_inner<S: Simd>(simd: S, src: Source<'_>, dst: Target<'_>) {
    const ALPHA: u32 = u32::from_ne_bytes([0, 0, 0, 255]);

    for_each_pixel_chunk([src], dst, {
        #[inline(always)]
        |[source]: [&[u8; 32]; 1], output: &mut [u8; 32]| {
            let pixels: u32x8<S> = u8x32::from_slice(simd, source).bitcast();
            (pixels & ALPHA).bitcast::<u8x32<S>>().store_slice(output);
        }
    });
}
