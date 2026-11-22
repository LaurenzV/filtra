# filtra_kernels

[![crates.io](https://img.shields.io/crates/v/filtra_kernels.svg)](https://crates.io/crates/filtra_kernels)
[![docs.rs](https://img.shields.io/docsrs/filtra_kernels.svg)](https://docs.rs/filtra_kernels)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)

Filter effect kernels on premultiplied RGBA8 pixmaps.

This crate provides SIMD-accelerated CPU kernels for each filter primitive defined in
[Filter Effects Module Level 1](https://drafts.fxtf.org/filter-effects-1/).

On WebAssembly, the kernels only use SIMD when compiled with the `simd128` target feature, for
example with `RUSTFLAGS="-C target-feature=+simd128"`.

## Features

Each filter primitive that has a kernel has a feature of the same name in kebab case, such as
`gaussian-blur`. `all`, the default, enables every kernel, and `css-functions` enables the ones that
filter functions need.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
