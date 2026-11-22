//! Pixel-level types and algorithms shared by the crates of filtra.
//!
//! This is an internal crate and not meant for external consumption.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]

#[cfg(feature = "gaussian-blur")]
pub mod blur;
mod color_space;
#[cfg(feature = "convolve-matrix")]
mod convolve_matrix;
#[cfg(feature = "lighting")]
mod lighting;
mod rect;
#[cfg(feature = "turbulence")]
mod turbulence;

pub use color_space::ColorSpace;
#[cfg(feature = "convolve-matrix")]
pub use convolve_matrix::Convolution;
#[cfg(feature = "lighting")]
pub use lighting::{Light, Lighting, Reflection};
pub use rect::IntRect;
#[cfg(feature = "turbulence")]
pub use turbulence::Turbulence;
