//! Filter effect kernels on premultiplied RGBA8 pixmaps.
//!
//! Kernels work in a single coordinate space, in which every input is placed with
//! [`Pixmap::source`] and the output with [`Pixmap::target`]. A kernel writes every pixel of its
//! target; pixels outside of an input are transparent black.

mod flood;
mod offset;
mod pixmap;
mod rect;

pub use flood::flood;
pub use offset::offset;
pub use pixmap::{Pixmap, Source, Target};
pub use rect::IntRect;
