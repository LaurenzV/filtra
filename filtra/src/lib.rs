//! Applies SVG and CSS filter effects to pixmaps.
//!
//! This crate provides a CPU renderer for arbitrary filter graphs, as defined in
//! [Filter Effects Module Level 1](https://drafts.fxtf.org/filter-effects-1/), with SIMD kernels
//! for NEON, AVX2 and WebAssembly.
//!
//! # Example
//!
//! The following example applies a small chain of filters to a 200×200 pixmap at 2× scale.
//!
//! ```
//! use filtra::types::{Filter, GaussianBlur, Input, Merge, Node, Offset, Primitive, Rect};
//! use filtra::{BufferPool, Pixmap, Transform};
//!
//! let mut filter = Filter::<()>::new(Rect::new(0.0, 0.0, 100.0, 100.0));
//! let blur = GaussianBlur { std_dev_x: 4.0, std_dev_y: 4.0, ..Default::default() };
//! let blurred = filter.push(Node::new(Primitive::GaussianBlur(blur), [Input::SourceAlpha]));
//! let offset = Offset { dx: 5.0, dy: 5.0 };
//! let shadow = filter.push(Node::new(Primitive::Offset(offset), [blurred]));
//! filter.push(Node::new(Primitive::Merge(Merge), [shadow, Input::SourceGraphic]));
//!
//! // The content, rendered at twice its size.
//! let mut pixmap = Pixmap::new(200, 200);
//! let transform = Transform::new(2.0, 2.0, 0.0, 0.0);
//! let mut pool = BufferPool::new(4);
//!
//! filtra::apply(&filter, pixmap.as_mut(), transform, &mut pool, |_, _, _, _| {});
//! ```

mod buffer_pool;
mod cpu;

pub use buffer_pool::BufferPool;
pub use filtra_kernels::{IntRect, Pixmap, PixmapMut};
pub use filtra_plan::Transform;
pub use filtra_types as types;

/// Applies `filter` to `pixmap` in place, with `transform` mapping user space to its pixels.
pub fn apply<I>(
    filter: &types::Filter<I>,
    pixmap: PixmapMut<'_>,
    transform: Transform,
    pool: &mut BufferPool,
    mut render_image: impl FnMut(&I, types::Rect, IntRect, &mut [u8]),
) {
    let bounds = pixmap.bounds();
    let bounds = filtra_plan::IntRect::new(bounds.x0, bounds.y0, bounds.x1, bounds.y1);
    let plan = filtra_plan::plan(filter, transform, bounds);
    let images: Vec<(&I, types::Rect)> = (filter.nodes().iter())
        .zip(filter.subregions())
        .filter_map(|(node, subregion)| match &node.primitive {
            types::Primitive::Image(image) => Some((image, subregion)),
            _ => None,
        })
        .collect();

    cpu::execute(&plan, pixmap, pool, &mut |image, rect, data| {
        let (image, subregion) = images[image];
        render_image(image, subregion, rect, data);
    });
}
