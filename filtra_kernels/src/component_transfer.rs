use fearless_simd::{Level, Simd, dispatch, f32x8, i32x8, prelude::*};

use crate::simd::{for_each_pixel_chunk, pack, premultiply, unpack};
use crate::{Source, Target};

/// `feComponentTransfer`, with every function given as piecewise linear segments.
pub fn component_transfer(src: Source<'_>, dst: Target<'_>, functions: [&[[f32; 2]]; 4]) {
    dispatch!(Level::new(), simd => component_transfer_inner(simd, src, dst, functions));
}

#[inline(always)]
fn component_transfer_inner<S: Simd>(
    simd: S,
    src: Source<'_>,
    dst: Target<'_>,
    functions: [&[[f32; 2]]; 4],
) {
    let opaque = f32x8::splat(simd, 255.0);

    let remap = {
        #[inline(always)]
        |value: f32x8<S>, alpha: f32x8<S>, regions: &[[f32; 2]]| {
            let count = regions.len() as f32;
            // A quotient of whole numbers is exact on a region boundary, so values on one don't
            // fall into the region before it.
            let position = value * count / alpha;
            let region = position.min(count - 1.0).floor();
            let [start, end] = region_values(simd, regions, region);

            (position - region).mul_add(end - start, start)
        }
    };

    for_each_pixel_chunk([src], dst, {
        #[inline(always)]
        |[source]: [&[u8; 32]; 1], output: &mut [u8; 32]| {
            let [r, g, b, a] = unpack(simd, source);
            let divisor = a.max(1.0);
            let remapped = [
                remap(r, divisor, functions[0]),
                remap(g, divisor, functions[1]),
                remap(b, divisor, functions[2]),
                remap(a, opaque, functions[3]),
            ];

            pack(premultiply(remapped), output);
        }
    });
}

#[inline(always)]
fn region_values<S: Simd>(simd: S, regions: &[[f32; 2]], region: f32x8<S>) -> [f32x8<S>; 2] {
    if let [[start, end]] = *regions {
        return [f32x8::splat(simd, start), f32x8::splat(simd, end)];
    }

    let (mut starts, mut ends) = ([0.0; 8], [0.0; 8]);
    let indices = region.to_int::<i32x8<S>>();

    for ((start, end), index) in starts.iter_mut().zip(&mut ends).zip(indices.as_slice()) {
        [*start, *end] = regions[*index as usize];
    }

    [f32x8::simd_from(simd, starts), f32x8::simd_from(simd, ends)]
}
