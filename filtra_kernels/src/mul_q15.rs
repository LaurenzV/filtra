#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::{int16x8_t, vqrdmulhq_s16};
#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
use core::arch::wasm32::{i16x8_q15mulr_sat, v128};
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(any(
    target_arch = "aarch64",
    target_arch = "x86",
    target_arch = "x86_64",
    all(target_arch = "wasm32", target_feature = "simd128")
))]
use fearless_simd::Level;
use fearless_simd::{Simd, i16x8, i16x16, prelude::*};

// The product of signed Q15 numbers, rounded to nearest. `-32768 * -32768` overflows, with a
// result that depends on the level, so callers must avoid it.
pub(crate) trait MulQ15: Simd {
    fn mul_q15(self, a: i16x16<Self>, b: i16x16<Self>) -> i16x16<Self>;
}

impl<S: Simd> MulQ15 for S {
    // `dispatch!` hides the concrete level behind `impl Simd`, so the level is matched here
    // instead, which folds to a single arm for each instantiation.
    #[inline(always)]
    fn mul_q15(self, a: i16x16<Self>, b: i16x16<Self>) -> i16x16<Self> {
        match self.level() {
            #[cfg(target_arch = "aarch64")]
            Level::Neon(neon) => halves(self, a, b, {
                #[inline(always)]
                |a, b| neon_mul_q15(neon, a.into(), b.into()).simd_into(self)
            }),
            #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
            Level::WasmSimd128(wasm) => halves(self, a, b, {
                #[inline(always)]
                |a, b| wasm_mul_q15(wasm, a.into(), b.into()).simd_into(self)
            }),
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Level::Avx512(avx512) => avx512_mul_q15(avx512, a.into(), b.into()).simd_into(self),
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Level::Avx2(avx2) => avx2_mul_q15(avx2, a.into(), b.into()).simd_into(self),
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Level::Sse4_2(sse4_2) => halves(self, a, b, {
                #[inline(always)]
                |a, b| sse4_2_mul_q15(sse4_2, a.into(), b.into()).simd_into(self)
            }),
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Level::Sse2(sse2) => halves(self, a, b, {
                #[inline(always)]
                |a, b| sse2_mul_q15(sse2, a.into(), b.into()).simd_into(self)
            }),
            _ => {
                let (a, b) = (<[i16; 16]>::from(a), <[i16; 16]>::from(b));
                let product: [i16; 16] = core::array::from_fn(|i| {
                    ((i32::from(a[i]) * i32::from(b[i]) + (1 << 14)) >> 15) as i16
                });

                product.simd_into(self)
            }
        }
    }
}

#[cfg_attr(
    not(any(
        target_arch = "aarch64",
        target_arch = "x86",
        target_arch = "x86_64",
        all(target_arch = "wasm32", target_feature = "simd128")
    )),
    expect(dead_code, reason = "only SIMD levels multiply in halves")
)]
#[inline(always)]
fn halves<S: Simd>(
    simd: S,
    a: i16x16<S>,
    b: i16x16<S>,
    mul: impl Fn(i16x8<S>, i16x8<S>) -> i16x8<S>,
) -> i16x16<S> {
    let (a0, a1) = simd.split_i16x16(a);
    let (b0, b1) = simd.split_i16x16(b);

    simd.combine_i16x8(mul(a0, b0), mul(a1, b1))
}

#[cfg(target_arch = "aarch64")]
fearless_simd::kernel!(
    #[inline(always)]
    fn neon_mul_q15(neon: Neon, a: int16x8_t, b: int16x8_t) -> int16x8_t {
        vqrdmulhq_s16(a, b)
    }
);

#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
fearless_simd::kernel!(
    #[inline(always)]
    fn wasm_mul_q15(wasm: WasmSimd128, a: v128, b: v128) -> v128 {
        i16x8_q15mulr_sat(a, b)
    }
);

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fearless_simd::kernel!(
    #[inline(always)]
    fn avx512_mul_q15(avx512: Avx512, a: __m256i, b: __m256i) -> __m256i {
        _mm256_mulhrs_epi16(a, b)
    }
);

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fearless_simd::kernel!(
    #[inline(always)]
    fn avx2_mul_q15(avx2: Avx2, a: __m256i, b: __m256i) -> __m256i {
        _mm256_mulhrs_epi16(a, b)
    }
);

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fearless_simd::kernel!(
    #[inline(always)]
    fn sse4_2_mul_q15(sse4_2: Sse4_2, a: __m128i, b: __m128i) -> __m128i {
        _mm_mulhrs_epi16(a, b)
    }
);

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fearless_simd::kernel!(
    #[inline(always)]
    fn sse2_mul_q15(sse2: Sse2, a: __m128i, b: __m128i) -> __m128i {
        // SSE2 has no `pmulhrsw`, so the result is assembled from the halves of the 32-bit
        // product: bits 15 to 30, plus bit 14 for the rounding.
        let (low, high) = (_mm_mullo_epi16(a, b), _mm_mulhi_epi16(a, b));
        let truncated = _mm_or_si128(_mm_slli_epi16::<1>(high), _mm_srli_epi16::<15>(low));
        let round = _mm_and_si128(_mm_srli_epi16::<14>(low), _mm_set1_epi16(1));

        _mm_add_epi16(truncated, round)
    }
);
