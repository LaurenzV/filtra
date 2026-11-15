use fearless_simd::{Level, Simd, dispatch, u16x16};

use crate::pixmap::segments;
use crate::simd::{Wide, combine, div255, for_each_chunk};
use crate::{Source, Target};

/// Composites `inputs` on top of each other with source over, the first one at the bottom.
pub fn merge(inputs: &[Source<'_>], dst: Target<'_>) {
    dispatch!(Level::new(), simd => merge_inner(simd, inputs, dst));
}

#[inline(always)]
fn merge_inner<S: Simd>(simd: S, inputs: &[Source<'_>], mut dst: Target<'_>) {
    let rect = dst.rect();
    let mut rows = Vec::with_capacity(inputs.len());

    let over = {
        #[inline(always)]
        |cs: u16x16<S>, sa: u16x16<S>, cb: u16x16<S>, _: u16x16<S>| cs + div255(cb * (255 - sa))
    };

    for (y, row) in dst.rows_mut() {
        for (x0, x1) in segments(rect.x0, rect.x1, inputs.iter().map(|input| input.rect())) {
            let output = &mut row[(x0 - rect.x0) as usize * 4..(x1 - rect.x0) as usize * 4];

            // Inputs that don't cover the segment are transparent, which source over leaves out.
            rows.clear();
            rows.extend(inputs.iter().filter_map(|input| input.row(y, x0, x1)));

            match rows.as_slice() {
                [] => output.fill(0),
                [first, rest @ ..] => {
                    output.copy_from_slice(first);

                    for source in rest {
                        for_each_chunk([Some(*source)], output, {
                            #[inline(always)]
                            |[source]: [&[u8; 32]; 1], output: &mut [u8; 32]| {
                                let backdrop = Wide::load(simd, output);
                                combine(Wide::load(simd, source), backdrop, over).store(output);
                            }
                        });
                    }
                }
            }
        }
    }
}
