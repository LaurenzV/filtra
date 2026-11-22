# filtra

[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)
[![CI](https://img.shields.io/github/actions/workflow/status/LaurenzV/filtra/ci.yml?logo=github&label=CI)](https://github.com/LaurenzV/filtra/actions)

A family of crates for describing and applying SVG and CSS filter effects, implemented from
[Filter Effects Module Level 1](https://drafts.fxtf.org/filter-effects-1/),
[Compositing and Blending Level 1](https://drafts.fxtf.org/compositing-1/) and
[CSS Color 4](https://drafts.csswg.org/css-color-4/).

The functionality is split across several crates, so that you can depend on only what you need.
To describe filter graphs without rendering them, `filtra_types` is enough. To apply individual
filters to pixel buffers on the CPU, depend on `filtra_kernels` directly, with only the features
you need enabled.

## Crates

| Crate | | |
|---|---|---|
| [`filtra`](filtra) | Applies filters to pixmaps on the CPU. | [![crates.io](https://img.shields.io/crates/v/filtra.svg)](https://crates.io/crates/filtra) [![docs.rs](https://img.shields.io/docsrs/filtra.svg)](https://docs.rs/filtra) |
| [`filtra_types`](filtra_types) | Types describing filter graphs. | [![crates.io](https://img.shields.io/crates/v/filtra_types.svg)](https://crates.io/crates/filtra_types) [![docs.rs](https://img.shields.io/docsrs/filtra_types.svg)](https://docs.rs/filtra_types) |
| [`filtra_core`](filtra_core) | Internal: pixel-level types and algorithms shared by the other crates. | [![crates.io](https://img.shields.io/crates/v/filtra_core.svg)](https://crates.io/crates/filtra_core) [![docs.rs](https://img.shields.io/docsrs/filtra_core.svg)](https://docs.rs/filtra_core) |
| [`filtra_kernels`](filtra_kernels) | SIMD CPU kernels for each filter primitive. | [![crates.io](https://img.shields.io/crates/v/filtra_kernels.svg)](https://crates.io/crates/filtra_kernels) [![docs.rs](https://img.shields.io/docsrs/filtra_kernels.svg)](https://docs.rs/filtra_kernels) |

## Notes

- The CPU implementation is essentially complete. The `BackgroundImage`, `BackgroundAlpha`,
  `FillPaint` and `StrokePaint` inputs, `edgeMode` on `feGaussianBlur`, and `kernelUnitLength`
  are not supported and unlikely to be soon.
- The CPU kernels favor speed over full accuracy. Images are stored as 8-bit premultiplied RGBA,
  and many filters compute with integers instead of floats, at the cost of more rounding error.
  Additionally providing a full 32-bit float pipeline is not planned.
- Support for GPU backends is planned.

## Minimum supported Rust version

1.89.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
