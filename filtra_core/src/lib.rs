//! Pixel-level types and algorithms shared by the crates of filtra.
//!
//! This is an internal crate and not meant for external consumption.

#![forbid(unsafe_code)]

#[cfg(feature = "gaussian-blur")]
pub mod blur;
mod color_space;
#[cfg(feature = "lighting")]
mod lighting;
mod rect;

pub use color_space::ColorSpace;
#[cfg(feature = "lighting")]
pub use lighting::{Light, Reflection};
pub use rect::IntRect;
