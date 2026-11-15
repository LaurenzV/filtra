//! Filter effect kernels on premultiplied RGBA8 pixmaps.
//!
//! Kernels work in a single coordinate space, in which the output is placed with
//! [`Pixmap::target`]. A kernel writes every pixel of its target.

mod flood;
mod pixmap;
mod rect;

pub use flood::flood;
pub use pixmap::{Pixmap, Target};
pub use rect::IntRect;
