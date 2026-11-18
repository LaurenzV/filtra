//! Filter effect kernels on premultiplied RGBA8 pixmaps.
//!
//! Kernels work in a single coordinate space, in which every input is placed with
//! [`Pixmap::source`] and the output with [`Pixmap::target`]. A kernel writes every pixel of its
//! target; pixels outside of an input are transparent black.

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
#[cfg(feature = "displacement-map")]
mod displacement_map;
mod flood;
#[cfg(any(feature = "diffuse-lighting", feature = "specular-lighting"))]
mod lighting;
#[cfg(feature = "merge")]
mod merge;
#[cfg(feature = "morphology")]
mod morphology;
mod offset;
// Some of the shared helpers go unused when only some primitives are enabled.
#[cfg_attr(not(feature = "all"), allow(dead_code, unused_imports))]
mod pixmap;
mod rect;
#[cfg_attr(not(feature = "all"), allow(dead_code, unused_imports))]
mod simd;
#[cfg(feature = "tile")]
mod tile;
#[cfg(feature = "turbulence")]
mod turbulence;

pub use alpha::alpha;
#[cfg(feature = "composite")]
pub use arithmetic::arithmetic;
#[cfg(feature = "blend")]
pub use blend::{BlendMode, blend};
#[cfg(feature = "gaussian-blur")]
pub use blur::gaussian_blur;
#[cfg(feature = "color-matrix")]
pub use color_matrix::color_matrix;
pub use color_space::{ColorSpace, convert};
#[cfg(feature = "component-transfer")]
pub use component_transfer::component_transfer;
#[cfg(feature = "composite")]
pub use composite::{PorterDuff, composite};
#[cfg(feature = "convolve-matrix")]
pub use convolve_matrix::{Convolution, EdgeMode, convolve_matrix};
#[cfg(feature = "displacement-map")]
pub use displacement_map::{ColorChannel, displacement_map};
pub use flood::flood;
#[cfg(any(feature = "diffuse-lighting", feature = "specular-lighting"))]
pub use lighting::{Light, Lighting, Reflection, lighting};
#[cfg(feature = "merge")]
pub use merge::merge;
#[cfg(feature = "morphology")]
pub use morphology::{MorphologyOperator, morphology};
pub use offset::offset;
pub use pixmap::{Pixmap, Source, Target};
pub use rect::IntRect;
#[cfg(feature = "tile")]
pub use tile::tile;
#[cfg(feature = "turbulence")]
pub use turbulence::{Turbulence, turbulence};

// TODO: In-place variants of kernels that compute each pixel from the same pixel of their input,
// for steps that may overwrite it.
