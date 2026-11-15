//! Filter effect kernels on premultiplied RGBA8 pixmaps.
//!
//! Kernels work in a single coordinate space, in which every input is placed with
//! [`Pixmap::source`] and the output with [`Pixmap::target`]. A kernel writes every pixel of its
//! target; pixels outside of an input are transparent black.

mod alpha;
mod arithmetic;
mod color_matrix;
mod color_space;
mod component_transfer;
mod composite;
mod flood;
mod merge;
mod offset;
mod pixmap;
mod rect;
mod simd;
mod tile;

pub use alpha::alpha;
pub use arithmetic::arithmetic;
pub use color_matrix::color_matrix;
pub use color_space::{ColorSpace, convert};
pub use component_transfer::component_transfer;
pub use composite::{PorterDuff, composite};
pub use flood::flood;
pub use merge::merge;
pub use offset::offset;
pub use pixmap::{Pixmap, Source, Target};
pub use rect::IntRect;
pub use tile::tile;

// TODO: In-place variants of kernels that compute each pixel from the same pixel of their input,
// for steps that may overwrite it.
