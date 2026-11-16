use fearless_simd::{Simd, SimdCvtFloat, f32x8, i32x8, prelude::*, u8x32, u16x16, u32x8};

// Every byte stays in the 32-bit lane of its pixel, which avoids shuffles on x86.
#[derive(Clone, Copy)]
pub(crate) struct Wide<S: Simd> {
    pub(crate) even: u16x16<S>,
    pub(crate) odd: u16x16<S>,
}

impl<S: Simd> Wide<S> {
    #[inline(always)]
    pub(crate) fn load(simd: S, bytes: &[u8; 32]) -> Self {
        Self::from_pixels(u8x32::from_slice(simd, bytes))
    }

    #[inline(always)]
    pub(crate) fn from_pixels(pixels: u8x32<S>) -> Self {
        let pixels: u16x16<S> = pixels.bitcast();
        Self {
            even: pixels & 0xff,
            odd: pixels >> 8,
        }
    }

    #[inline(always)]
    pub(crate) fn pixels(self) -> u8x32<S> {
        (self.even | (self.odd << 8)).bitcast()
    }

    #[inline(always)]
    pub(crate) fn alpha(self) -> u16x16<S> {
        let odd: u32x8<S> = self.odd.bitcast();
        ((odd >> 16) | (odd & 0xffff_0000)).bitcast()
    }

    #[inline(always)]
    pub(crate) fn store(self, bytes: &mut [u8; 32]) {
        self.pixels().store_slice(bytes);
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
    let (full, tail) = output.as_chunks_mut::<32>();
    // A missing input has no chunks, so its padded chunk stays transparent.
    let inputs = core::array::from_fn::<_, N, _>(|k| {
        inputs[k].map_or((&[][..], &[][..]), |input| input.as_chunks::<32>())
    });
    let mut padded_inputs = [[0; 32]; N];
    let mut padded_output = [0; 32];

    if !tail.is_empty() {
        for (padded, (_, rest)) in padded_inputs.iter_mut().zip(&inputs) {
            padded[..rest.len()].copy_from_slice(rest);
        }

        padded_output[..tail.len()].copy_from_slice(tail);
    }

    for i in 0..full.len() + usize::from(!tail.is_empty()) {
        let chunks = core::array::from_fn(|k| inputs[k].0.get(i).unwrap_or(&padded_inputs[k]));
        f(chunks, full.get_mut(i).unwrap_or(&mut padded_output));
    }

    if !tail.is_empty() {
        tail.copy_from_slice(&padded_output[..tail.len()]);
    }
}

#[inline(always)]
pub(crate) fn unpack<S: Simd>(simd: S, bytes: &[u8; 32]) -> [f32x8<S>; 4] {
    let pixels: u32x8<S> = u8x32::from_slice(simd, bytes).bitcast();
    let channel = {
        #[inline(always)]
        |shift: u32| f32x8::float_from(((pixels >> shift) & 0xff).bitcast::<i32x8<S>>())
    };

    [channel(0), channel(8), channel(16), channel(24)]
}

#[inline(always)]
pub(crate) fn pack<S: Simd>(channels: [f32x8<S>; 4], bytes: &mut [u8; 32]) {
    let channel = {
        #[inline(always)]
        |value: f32x8<S>, shift: u32| {
            (value + 0.5).to_int::<i32x8<S>>().bitcast::<u32x8<S>>() << shift
        }
    };
    let [r, g, b, a] = channels;

    (channel(r, 0) | channel(g, 8) | channel(b, 16) | channel(a, 24))
        .bitcast::<u8x32<S>>()
        .store_slice(bytes);
}

#[inline(always)]
pub(crate) fn unpremultiply<S: Simd>(channels: [f32x8<S>; 4]) -> [f32x8<S>; 4] {
    let [r, g, b, a] = channels;
    let scale = 1.0 / a.max(1.0);

    [r * scale, g * scale, b * scale, a * (1.0 / 255.0)]
}

#[inline(always)]
pub(crate) fn premultiply<S: Simd>(channels: [f32x8<S>; 4]) -> [f32x8<S>; 4] {
    let [r, g, b, a] = channels;
    let alpha = a.max(0.0).min(1.0) * 255.0;
    let color = {
        #[inline(always)]
        |value: f32x8<S>| value.max(0.0).min(1.0) * alpha
    };

    [color(r), color(g), color(b), alpha]
}
