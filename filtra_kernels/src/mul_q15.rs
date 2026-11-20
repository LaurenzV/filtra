#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::{int16x8_t, vqrdmulhq_s16};
#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
use core::arch::wasm32::{i16x8_q15mulr_sat, v128};
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(not(any(
    all(target_arch = "aarch64", target_feature = "neon"),
    all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2",
        target_feature = "fxsr"
    ),
    all(target_arch = "wasm32", target_feature = "simd128")
)))]
use fearless_simd::Fallback;
#[cfg(target_arch = "aarch64")]
use fearless_simd::Neon;
#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
use fearless_simd::WasmSimd128;
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "x86",
    target_arch = "x86_64",
    all(target_arch = "wasm32", target_feature = "simd128")
))]
use fearless_simd::i16x8;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use fearless_simd::{Avx2, Sse2, Sse4_2};
use fearless_simd::{Simd, i16x16, prelude::*};

// The product of signed Q15 numbers, rounded to nearest. `-32768 * -32768` overflows, with a
// result that depends on the level, so callers must avoid it.
pub(crate) trait MulQ15: Simd {
    fn mul_q15(self, a: i16x16<Self>, b: i16x16<Self>) -> i16x16<Self>;
}

#[cfg(target_arch = "aarch64")]
impl MulQ15 for Neon {
    #[inline(always)]
    fn mul_q15(self, a: i16x16<Self>, b: i16x16<Self>) -> i16x16<Self> {
        fearless_simd::kernel!(
            #[inline(always)]
            fn mul(neon: Neon, a: int16x8_t, b: int16x8_t) -> int16x8_t {
                vqrdmulhq_s16(a, b)
            }
        );

        halves(self, a, b, mul)
    }
}

#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
impl MulQ15 for WasmSimd128 {
    #[inline(always)]
    fn mul_q15(self, a: i16x16<Self>, b: i16x16<Self>) -> i16x16<Self> {
        fearless_simd::kernel!(
            #[inline(always)]
            fn mul(wasm: WasmSimd128, a: v128, b: v128) -> v128 {
                i16x8_q15mulr_sat(a, b)
            }
        );

        halves(self, a, b, mul)
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl MulQ15 for Avx2 {
    #[inline(always)]
    fn mul_q15(self, a: i16x16<Self>, b: i16x16<Self>) -> i16x16<Self> {
        fearless_simd::kernel!(
            #[inline(always)]
            fn mul(avx2: Avx2, a: __m256i, b: __m256i) -> __m256i {
                _mm256_mulhrs_epi16(a, b)
            }
        );

        mul(self, a.into(), b.into()).simd_into(self)
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl MulQ15 for Sse4_2 {
    #[inline(always)]
    fn mul_q15(self, a: i16x16<Self>, b: i16x16<Self>) -> i16x16<Self> {
        fearless_simd::kernel!(
            #[inline(always)]
            fn mul(sse4_2: Sse4_2, a: __m128i, b: __m128i) -> __m128i {
                _mm_mulhrs_epi16(a, b)
            }
        );

        halves(self, a, b, mul)
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl MulQ15 for Sse2 {
    #[inline(always)]
    fn mul_q15(self, a: i16x16<Self>, b: i16x16<Self>) -> i16x16<Self> {
        fearless_simd::kernel!(
            #[inline(always)]
            fn mul(sse2: Sse2, a: __m128i, b: __m128i) -> __m128i {
                // SSE2 has no `pmulhrsw`, so the result is assembled from the halves of the
                // 32-bit product: bits 15 to 30, plus bit 14 for the rounding.
                let (low, high) = (_mm_mullo_epi16(a, b), _mm_mulhi_epi16(a, b));
                let truncated = _mm_or_si128(_mm_slli_epi16::<1>(high), _mm_srli_epi16::<15>(low));
                let round = _mm_and_si128(_mm_srli_epi16::<14>(low), _mm_set1_epi16(1));

                _mm_add_epi16(truncated, round)
            }
        );

        halves(self, a, b, mul)
    }
}

#[cfg(not(any(
    all(target_arch = "aarch64", target_feature = "neon"),
    all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2",
        target_feature = "fxsr"
    ),
    all(target_arch = "wasm32", target_feature = "simd128")
)))]
impl MulQ15 for Fallback {
    #[inline(always)]
    fn mul_q15(self, a: i16x16<Self>, b: i16x16<Self>) -> i16x16<Self> {
        let (a, b) = (<[i16; 16]>::from(a), <[i16; 16]>::from(b));
        let product: [i16; 16] = core::array::from_fn(|i| {
            ((i32::from(a[i]) * i32::from(b[i]) + (1 << 14)) >> 15) as i16
        });

        product.simd_into(self)
    }
}

// Applies `mul` to both 128-bit halves, for levels whose 256-bit vectors are pairs of 128-bit ones.
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "x86",
    target_arch = "x86_64",
    all(target_arch = "wasm32", target_feature = "simd128")
))]
#[inline(always)]
fn halves<S: Simd, R: From<i16x8<S>> + SimdInto<i16x8<S>, S>>(
    simd: S,
    a: i16x16<S>,
    b: i16x16<S>,
    mul: impl Fn(S, R, R) -> R,
) -> i16x16<S> {
    let (a0, a1) = simd.split_i16x16(a);
    let (b0, b1) = simd.split_i16x16(b);
    let mul = {
        #[inline(always)]
        |a: i16x8<S>, b: i16x8<S>| mul(simd, a.into(), b.into()).simd_into(simd)
    };

    simd.combine_i16x8(mul(a0, b0), mul(a1, b1))
}
