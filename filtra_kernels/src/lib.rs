//! Filter effect kernels on premultiplied RGBA8 pixmaps.
//!
//! This crate provides SIMD-accelerated CPU kernels for each filter primitive defined in
//! [Filter Effects Module Level 1](https://drafts.fxtf.org/filter-effects-1/).
//!
//! Kernels work in a single coordinate space, in which every input is placed with
//! [`Source::new`] and the output with [`Target::new`]. A kernel writes every pixel of its target;
//! pixels outside of an input are transparent black.

#![forbid(unsafe_code)]

mod alpha;
#[cfg(feature = "composite")]
mod arithmetic;
#[cfg(feature = "blend")]
mod blend;
#[cfg(feature = "gaussian-blur")]
mod blur;
#[cfg(feature = "color-matrix")]
mod color_matrix;
mod color_space;
#[cfg(feature = "component-transfer")]
mod component_transfer;
#[cfg(feature = "composite")]
mod composite;
#[cfg(feature = "convolve-matrix")]
mod convolve_matrix;
mod dispatch;
#[cfg(feature = "displacement-map")]
mod displacement_map;
mod flood;
#[cfg(any(feature = "diffuse-lighting", feature = "specular-lighting"))]
mod lighting;
#[cfg(feature = "merge")]
mod merge;
#[cfg(feature = "morphology")]
mod morphology;
#[cfg(feature = "gaussian-blur")]
mod mul_q15;
mod offset;
// Some of the shared helpers go unused when only some primitives are enabled.
#[cfg_attr(not(feature = "all"), allow(dead_code, unused_imports))]
mod pixmap;
#[cfg_attr(not(feature = "all"), allow(dead_code, unused_imports))]
mod simd;
#[cfg(feature = "tile")]
mod tile;
#[cfg(feature = "turbulence")]
mod turbulence;

pub use alpha::alpha;
#[cfg(feature = "blend")]
pub use blend::blend;
#[cfg(feature = "gaussian-blur")]
pub use blur::gaussian_blur;
#[cfg(feature = "color-matrix")]
pub use color_matrix::color_matrix;
pub use color_space::convert;
#[cfg(feature = "component-transfer")]
pub use component_transfer::component_transfer;
#[cfg(feature = "composite")]
pub use composite::composite;
#[cfg(feature = "convolve-matrix")]
pub use convolve_matrix::convolve_matrix;
#[cfg(feature = "displacement-map")]
pub use displacement_map::displacement_map;
pub use flood::flood;
#[cfg(any(feature = "diffuse-lighting", feature = "specular-lighting"))]
pub use lighting::lighting;
#[cfg(feature = "merge")]
pub use merge::merge;
#[cfg(feature = "morphology")]
pub use morphology::morphology;
pub use offset::offset;
pub use pixmap::{Source, Target};
#[cfg(feature = "tile")]
pub use tile::tile;
#[cfg(feature = "turbulence")]
pub use turbulence::turbulence;

#[cfg(feature = "convolve-matrix")]
pub use filtra_core::Convolution;
#[cfg(feature = "turbulence")]
pub use filtra_core::Turbulence;
pub use filtra_core::{ColorSpace, IntRect};
#[cfg(any(feature = "diffuse-lighting", feature = "specular-lighting"))]
pub use filtra_core::{Light, Lighting, Reflection};
#[cfg(feature = "blend")]
pub use filtra_types::BlendMode;
#[cfg(feature = "displacement-map")]
pub use filtra_types::ColorChannel;
#[cfg(feature = "composite")]
pub use filtra_types::CompositeOperator;
#[cfg(feature = "convolve-matrix")]
pub use filtra_types::EdgeMode;
#[cfg(feature = "morphology")]
pub use filtra_types::MorphologyOperator;
