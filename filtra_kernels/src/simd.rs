use fearless_simd::{Simd, prelude::*, u8x32, u16x16, u32x8};

// Every byte stays in the 32-bit lane of its pixel, which avoids shuffles on x86.
#[derive(Clone, Copy)]
pub(crate) struct Wide<S: Simd> {
    pub(crate) even: u16x16<S>,
    pub(crate) odd: u16x16<S>,
}

impl<S: Simd> Wide<S> {
    #[inline(always)]
    pub(crate) fn load(simd: S, bytes: &[u8; 32]) -> Self {
        let pixels: u16x16<S> = u8x32::from_slice(simd, bytes).bitcast();
        Self {
            even: pixels & 0xff,
            odd: pixels >> 8,
        }
    }

    #[inline(always)]
    pub(crate) fn alpha(self) -> u16x16<S> {
        let odd: u32x8<S> = self.odd.bitcast();
        ((odd >> 16) | (odd & 0xffff_0000)).bitcast()
    }

    #[inline(always)]
    pub(crate) fn store(self, bytes: &mut [u8; 32]) {
        (self.even | (self.odd << 8))
            .bitcast::<u8x32<S>>()
            .store_slice(bytes);
    }
}

#[inline(always)]
pub(crate) fn combine<S: Simd>(
    source: Wide<S>,
    backdrop: Wide<S>,
    f: impl Fn(u16x16<S>, u16x16<S>, u16x16<S>, u16x16<S>) -> u16x16<S>,
) -> Wide<S> {
    let (source_alpha, backdrop_alpha) = (source.alpha(), backdrop.alpha());

    Wide {
        even: f(source.even, source_alpha, backdrop.even, backdrop_alpha),
        odd: f(source.odd, source_alpha, backdrop.odd, backdrop_alpha),
    }
}

// Exact for `x` up to 255².
#[inline(always)]
pub(crate) fn div255<S: Simd>(x: u16x16<S>) -> u16x16<S> {
    let x = x + 128;
    (x + (x >> 8)) >> 8
}

#[inline(always)]
pub(crate) fn for_each_chunk<const N: usize>(
    inputs: [Option<&[u8]>; N],
    output: &mut [u8],
    mut f: impl FnMut([&[u8; 32]; N], &mut [u8; 32]),
) {
    const TRANSPARENT: [u8; 32] = [0; 32];

    let len = output.len() / 32 * 32;
    let (full, tail) = output.split_at_mut(len);

    for (i, output) in full.as_chunks_mut::<32>().0.iter_mut().enumerate() {
        let chunks = inputs.map(|input| match input {
            Some(input) => input[i * 32..][..32].try_into().unwrap(),
            None => &TRANSPARENT,
        });

        f(chunks, output);
    }

    if !tail.is_empty() {
        let padded = inputs.map(|input| {
            let mut chunk = [0; 32];

            if let Some(input) = input {
                chunk[..tail.len()].copy_from_slice(&input[len..]);
            }

            chunk
        });

        let mut chunk = [0; 32];
        chunk[..tail.len()].copy_from_slice(tail);
        f(padded.each_ref(), &mut chunk);
        tail.copy_from_slice(&chunk[..tail.len()]);
    }
}
