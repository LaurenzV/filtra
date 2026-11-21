//! The shaders of filtra's GPU passes.
//!
//! Every pass draws [`VERTEX_WGSL`] with the fragment shader of its [`Shader`], in WGSL for wgpu or
//! GLSL ES 3.00 for WebGL2. A fragment shader reads its uniforms at `@group(0) @binding(0)` and its
//! inputs at the following bindings.

#![no_std]
#![forbid(unsafe_code)]

#[rustfmt::skip]
mod generated;

/// The vertex shader of every pass in WGSL.
pub const VERTEX_WGSL: &str = generated::VERTEX_WGSL;

/// The vertex shader of every pass in GLSL ES 3.00.
pub const VERTEX_GLSL: &str = generated::VERTEX_GLSL;

/// The fragment shader of a kind of pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Shader {
    /// The alpha channel of the input with black color.
    Alpha,
    /// A blend of two inputs with a blend mode.
    Blend,
    /// A 4×5 color matrix applied to the unpremultiplied input.
    ColorMatrix,
    /// A Porter-Duff, lighter or arithmetic composite of two inputs.
    Composite,
    /// The input converted between sRGB and linearRGB.
    Convert,
    /// The input with its pixels moved by the channels of a map.
    DisplacementMap,
    /// A uniform color.
    Flood,
    /// The input halved along one axis with [1, 3, 3, 1] / 8.
    Downsample,
    /// A Gaussian of up to 13 taps along one axis.
    GaussianBlur,
    /// Diffuse or specular lighting of the input's alpha as a height map.
    Lighting,
    /// Up to four inputs stacked with source over.
    Merge,
    /// The minimum or maximum of the input along one axis.
    Morphology,
    /// The input moved by whole pixels.
    Offset,
    /// A part of the input repeated.
    Tile,
    /// The input doubled along one axis with [1, 3] / 4 interpolation.
    Upsample,
}

impl Shader {
    /// The fragment shader in WGSL, with the entry point `main`.
    pub const fn wgsl(self) -> &'static str {
        generated::generated(self).wgsl
    }

    /// The fragment shader in GLSL ES 3.00.
    pub const fn glsl(self) -> &'static str {
        generated::generated(self).glsl
    }

    /// The name of the uniform block with the parameters in [`Shader::glsl`].
    pub const fn glsl_uniform_block(self) -> &'static str {
        generated::generated(self).glsl_uniform_block
    }

    /// The names of the samplers of the inputs in [`Shader::glsl`], in binding order.
    pub const fn glsl_textures(self) -> &'static [&'static str] {
        generated::generated(self).glsl_textures
    }
}
