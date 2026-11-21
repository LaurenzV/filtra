use fearless_simd::{Simd, prelude::*, u8x32, u16x16};
use filtra_types::CompositeOperator;

use crate::arithmetic::arithmetic;
use crate::dispatch::dispatch;
use crate::simd::{Wide, combine, div255, for_each_pixel_chunk, source_over};
use crate::{Source, Target};

/// `feComposite`.
pub fn composite(
    src: Source<'_>,
    backdrop: Source<'_>,
    dst: Target<'_>,
    operator: CompositeOperator,
) {
    dispatch!(simd => composite_inner(simd, src, backdrop, dst, operator));
}

#[inline(always)]
fn composite_inner<S: Simd>(
    simd: S,
    src: Source<'_>,
    backdrop: Source<'_>,
    dst: Target<'_>,
    operator: CompositeOperator,
) {
    let inputs = [src, backdrop];

    match operator {
        CompositeOperator::Lighter => lighter(simd, inputs, dst),
        CompositeOperator::Arithmetic { k1, k2, k3, k4 } => {
            arithmetic(simd, inputs, dst, [k1, k2, k3, k4]);
        }
        _ => porter_duff(simd, inputs, dst, {
            #[inline(always)]
            |cs, sa, cb, ba| match operator {
                CompositeOperator::In => div255(cs * ba),
                CompositeOperator::Out => div255(cs * (255 - ba)),
                CompositeOperator::Atop => div255(cs * ba + cb * (255 - sa)),
                CompositeOperator::Xor => div255(cs * (255 - ba) + cb * (255 - sa)),
                _ => source_over(cs, sa, cb, ba),
            }
        }),
    }
}

#[inline(always)]
fn lighter<S: Simd>(simd: S, inputs: [Source<'_>; 2], dst: Target<'_>) {
    for_each_pixel_chunk(inputs, dst, {
        #[inline(always)]
        |[s, b]: [&[u8; 32]; 2], output: &mut [u8; 32]| {
            u8x32::from_slice(simd, s)
                .saturating_add(u8x32::from_slice(simd, b))
                .store_slice(output);
        }
    });
}

#[inline(always)]
fn porter_duff<S: Simd>(
    simd: S,
    inputs: [Source<'_>; 2],
    dst: Target<'_>,
    f: impl Fn(u16x16<S>, u16x16<S>, u16x16<S>, u16x16<S>) -> u16x16<S> + Copy,
) {
    for_each_pixel_chunk(inputs, dst, {
        #[inline(always)]
        |[s, b]: [&[u8; 32]; 2], output: &mut [u8; 32]| {
            combine(Wide::load(simd, s), Wide::load(simd, b), f).store(output);
        }
    });
}
