use core::f32::consts::LN_2;

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

#[inline(always)]
pub(crate) fn pow<S: Simd>(base: f32x8<S>, exponent: f32) -> f32x8<S> {
    // `0⁰` is 1, while `0 × log2(0)` would be NaN.
    if exponent == 0.0 {
        return f32x8::splat(base.simd, 1.0);
    }

    exp2(log2(base) * exponent)
}

// Writes `x = m · 2^k` with `m` in [√½, √2), where `t = (m − 1) / (m + 1)` stays within ±0.172, so
// that the series `log2(m) = 2 / ln 2 · (t + t³/3 + t⁵/5 + …)` is exact to about 1e-9 after `t⁹`.
#[inline(always)]
fn log2<S: Simd>(x: f32x8<S>) -> f32x8<S> {
    let simd = x.simd;
    let bits: i32x8<S> = x.to_bits().bitcast();
    let k = (bits - 0x3f35_04f3) >> 23;
    let m = f32x8::from_bits((bits - (k << 23)).bitcast());
    let t = (m - 1.0) / (m + 1.0);
    let t2 = t * t;
    let c = {
        #[inline(always)]
        |n: f32| f32x8::splat(simd, 2.0 / (n * LN_2))
    };
    let series = t2.mul_add(c(9.0), c(7.0));
    let series = t2.mul_add(series, c(5.0));
    let series = t2.mul_add(series, c(3.0));
    let series = t2.mul_add(series, c(1.0));
    let log = t.mul_add(series, f32x8::float_from(k));

    x.simd_gt(0.0)
        .select(log, f32x8::splat(simd, f32::NEG_INFINITY))
}

// Writes `x = n + f` with a whole `n` and `|f| ≤ ½`, and takes `2^f = e^(f ln 2)` from its Taylor
// series, which is exact to about 1e-8 after the seventh power.
#[inline(always)]
fn exp2<S: Simd>(x: f32x8<S>) -> f32x8<S> {
    let simd = x.simd;
    let x = x.max(-126.0).min(126.0);
    let n = x.round();
    let f = (x - n) * LN_2;
    let mut series = f32x8::splat(simd, 1.0);

    for k in (1..=7).rev() {
        series = (f * (1.0 / k as f32)).mul_add(series, 1.0);
    }

    let scale = f32x8::from_bits(((n.to_int::<i32x8<S>>() + 127) << 23).bitcast());

    series * scale
}

#[cfg(test)]
mod tests {
    use fearless_simd::{Level, Simd, dispatch, f32x8, prelude::*};

    #[test]
    fn pow_matches_f64() {
        dispatch!(Level::new(), simd => check_pow(simd));
    }

    fn check_pow<S: Simd>(simd: S) {
        let exponents = [
            -2.0, -1.0, 0.0, 0.25, 0.5, 1.0, 2.0, 7.5, 20.0, 128.0, 1000.0,
        ];
        let bases = (0..=4096)
            .map(|i| i as f32 / 4096.0)
            .chain([1e-30, 1e-10, 1.5, 3.0, 1e10]);

        for exponent in exponents {
            for base in bases.clone() {
                let actual = super::pow(f32x8::splat(simd, base), exponent).as_slice()[0];
                let expected = f64::from(base).powf(f64::from(exponent));
                let expected = if exponent == 0.0 { 1.0 } else { expected };
                let error = (f64::from(actual) - expected).abs();

                assert!(
                    error <= 2e-5 * expected.max(1.0 / 255.0) || (expected > 1e30 && actual > 1e30),
                    "{base}^{exponent}: got {actual}, expected {expected}"
                );
            }
        }
    }
}
