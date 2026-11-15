//! Filter effect kernels on premultiplied RGBA8 pixmaps.
//!
//! Kernels work in a single coordinate space, in which the output is placed with
//! [`Pixmap::target`]. A kernel writes every pixel of its target.

mod pixmap;
mod rect;

pub use pixmap::{Pixmap, Target};
pub use rect::IntRect;
