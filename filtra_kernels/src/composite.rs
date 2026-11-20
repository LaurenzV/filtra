use fearless_simd::{Level, Simd, dispatch, prelude::*, u8x32, u16x16};
use filtra_types::CompositeOperator;

use crate::arithmetic::arithmetic;
use crate::simd::{Wide, combine, div255, for_each_pixel_chunk};
use crate::{Source, Target};

/// `feComposite`.
pub fn composite(
    src: Source<'_>,
    backdrop: Source<'_>,
    dst: Target<'_>,
    operator: CompositeOperator,
) {
    dispatch!(Level::new(), simd => composite_inner(simd, src, backdrop, dst, operator));
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
        CompositeOperator::Over => over(simd, inputs, dst),
        CompositeOperator::In => in_(simd, inputs, dst),
        CompositeOperator::Out => out(simd, inputs, dst),
        CompositeOperator::Atop => atop(simd, inputs, dst),
        CompositeOperator::Xor => xor(simd, inputs, dst),
        CompositeOperator::Lighter => lighter(simd, inputs, dst),
        CompositeOperator::Arithmetic { k1, k2, k3, k4 } => {
            arithmetic(simd, inputs, dst, [k1, k2, k3, k4]);
        }
    }
}

macro_rules! porter_duff {
    ($name:ident, |$cs:ident, $sa:ident, $cb:ident, $ba:ident| $formula:expr) => {
        #[inline(always)]
        fn $name<S: Simd>(simd: S, inputs: [Source<'_>; 2], dst: Target<'_>) {
            porter_duff(simd, inputs, dst, {
                #[inline(always)]
                #[allow(unused_variables, reason = "Not every operator uses every input.")]
                |$cs, $sa, $cb, $ba| $formula
            });
        }
    };
}

porter_duff!(over, |cs, sa, cb, ba| cs + div255(cb * (255 - sa)));
porter_duff!(in_, |cs, sa, cb, ba| div255(cs * ba));
porter_duff!(out, |cs, sa, cb, ba| div255(cs * (255 - ba)));
porter_duff!(atop, |cs, sa, cb, ba| div255(cs * ba + cb * (255 - sa)));
porter_duff!(xor, |cs, sa, cb, ba| div255(
    cs * (255 - ba) + cb * (255 - sa)
));

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
