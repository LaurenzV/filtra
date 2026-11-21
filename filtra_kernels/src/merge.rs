use fearless_simd::Simd;

use crate::dispatch::dispatch;
use crate::pixmap::for_each_segment;
use crate::simd::{Wide, combine, for_each_chunk, source_over};
use crate::{Source, Target};

/// `feMerge`.
pub fn merge(inputs: &[Source<'_>], dst: Target<'_>) {
    dispatch!(simd => merge_inner(simd, inputs, dst));
}

#[inline(always)]
fn merge_inner<S: Simd>(simd: S, inputs: &[Source<'_>], mut dst: Target<'_>) {
    let mut rows = Vec::with_capacity(inputs.len());

    for_each_segment(&mut dst, inputs.iter().map(Source::rect), {
        #[inline(always)]
        |y, x0, x1, output| {
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
                            |_, [source]: [&[u8; 32]; 1], output: &mut [u8; 32]| {
                                let backdrop = Wide::load(simd, output);
                                combine(Wide::load(simd, source), backdrop, source_over)
                                    .store(output);
                            }
                        });
                    }
                }
            }
        }
    });
}
