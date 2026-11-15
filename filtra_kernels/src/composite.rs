use fearless_simd::{Level, Simd, dispatch, prelude::*, u8x32, u16x16};

use crate::pixmap::segments;
use crate::simd::{Wide, combine, div255, for_each_chunk};
use crate::{Source, Target};

/// A Porter-Duff compositing operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PorterDuff {
    /// Source over destination.
    Over,
    /// Source in destination.
    In,
    /// Source out of destination.
    Out,
    /// Source atop destination.
    Atop,
    /// Source XOR destination.
    Xor,
    /// The sum of source and destination, clamped.
    Lighter,
}

/// Composites `src` onto `backdrop` with a Porter-Duff operator.
pub fn composite(src: Source<'_>, backdrop: Source<'_>, operator: PorterDuff, dst: Target<'_>) {
    dispatch!(Level::new(), simd => composite_inner(simd, src, backdrop, operator, dst));
}

#[inline(always)]
fn composite_inner<S: Simd>(
    simd: S,
    src: Source<'_>,
    backdrop: Source<'_>,
    operator: PorterDuff,
    mut dst: Target<'_>,
) {
    let rect = dst.rect();

    for (y, row) in dst.rows_mut() {
        for (x0, x1) in segments(rect.x0, rect.x1, [src.rect(), backdrop.rect()].into_iter()) {
            let output = &mut row[(x0 - rect.x0) as usize * 4..(x1 - rect.x0) as usize * 4];
            let inputs = [src.row(y, x0, x1), backdrop.row(y, x0, x1)];

            match operator {
                PorterDuff::Over => over(simd, inputs, output),
                PorterDuff::In => in_(simd, inputs, output),
                PorterDuff::Out => out(simd, inputs, output),
                PorterDuff::Atop => atop(simd, inputs, output),
                PorterDuff::Xor => xor(simd, inputs, output),
                PorterDuff::Lighter => lighter(simd, inputs, output),
            }
        }
    }
}

macro_rules! porter_duff {
    ($name:ident, |$cs:ident, $sa:ident, $cb:ident, $ba:ident| $formula:expr) => {
        #[inline(always)]
        fn $name<S: Simd>(simd: S, inputs: [Option<&[u8]>; 2], output: &mut [u8]) {
            porter_duff(simd, inputs, output, {
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
fn lighter<S: Simd>(simd: S, inputs: [Option<&[u8]>; 2], output: &mut [u8]) {
    for_each_chunk(inputs, output, {
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
    inputs: [Option<&[u8]>; 2],
    output: &mut [u8],
    f: impl Fn(u16x16<S>, u16x16<S>, u16x16<S>, u16x16<S>) -> u16x16<S> + Copy,
) {
    for_each_chunk(inputs, output, {
        #[inline(always)]
        |[s, b]: [&[u8; 32]; 2], output: &mut [u8; 32]| {
            combine(Wide::load(simd, s), Wide::load(simd, b), f).store(output);
        }
    });
}
