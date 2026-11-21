# filtra_shaders

[![crates.io](https://img.shields.io/crates/v/filtra_shaders.svg)](https://crates.io/crates/filtra_shaders)
[![docs.rs](https://img.shields.io/docsrs/filtra_shaders.svg)](https://docs.rs/filtra_shaders)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)

The shaders of filtra's GPU passes.

Every pass draws the shared vertex shader with the fragment shader of its `Shader`, in WGSL for
wgpu or GLSL ES 3.00 for WebGL2. A fragment shader reads its uniforms at `@group(0) @binding(0)`
and its inputs at the following bindings.

## Changing a shader

The shaders are written in [WESL](https://wesl-lang.dev) in `shaders`. After changing one, run
`cargo run -p filtra_shaders_gen` to update the linked and minified WGSL and GLSL in `generated`,
which is checked in so that this crate has no dependencies.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
