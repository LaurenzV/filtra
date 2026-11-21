// Like `fearless_simd::dispatch!`, but with fewer x86 copies of each kernel, which mostly add binary
// size: AVX-512 runs the AVX2 versions, since the kernels use 256-bit vectors, and SSE4.2 runs the
// SSE2 versions, which are only a few percent slower. Like `fearless_simd::dispatch!`, it honors
// `--cfg disable_dispatch_avx2`, which leaves only the SSE2 versions on x86. The kernels also get
// the concrete level type, which lets traits like `MulQ15` be implemented per level.
macro_rules! dispatch {
    ($simd:pat => $op:expr) => {{
        use fearless_simd::{Level, Simd};

        match Level::new() {
            #[cfg(target_arch = "aarch64")]
            Level::Neon(token) => token.vectorize(
                #[inline(always)]
                || {
                    let $simd = token;
                    $op
                },
            ),
            #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
            Level::WasmSimd128(token) => token.vectorize(
                #[inline(always)]
                || {
                    let $simd = token;
                    $op
                },
            ),
            #[cfg(all(
                any(target_arch = "x86", target_arch = "x86_64"),
                not(disable_dispatch_avx2)
            ))]
            level @ (Level::Avx2(_) | Level::Avx512(_)) => {
                let token = level.as_avx2().expect("AVX-512 implies AVX2");

                token.vectorize(
                    #[inline(always)]
                    || {
                        let $simd = token;
                        $op
                    },
                )
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
            Level::Fallback(token) => token.vectorize(
                #[inline(always)]
                || {
                    let $simd = token;
                    $op
                },
            ),
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            level => {
                let token = level
                    .as_sse2()
                    .expect("every x86 level but the fallback has SSE2");

                token.vectorize(
                    #[inline(always)]
                    || {
                        let $simd = token;
                        $op
                    },
                )
            }
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
            _ => unreachable!("`Level::new` only detects the levels above"),
        }
    }};
}

pub(crate) use dispatch;
