//! Applies SVG and CSS filter effects to pixmaps.
//!
//! This crate applies arbitrary filter graphs, as defined in
//! [Filter Effects Module Level 1](https://drafts.fxtf.org/filter-effects-1/), with any
//! [`Backend`]. It provides [`cpu::Backend`], with SIMD kernels for NEON, AVX2 and WebAssembly.
//!
//! # Example
//!
//! The following example applies a small chain of filters to a 200×200 pixmap at 2× scale.
//!
//! ```
//! use filtra::types::{Filter, GaussianBlur, Input, Merge, Node, Offset, Primitive, Rect};
//! use filtra::Transform;
//! use filtra::cpu::{self, PixmapMut};
//!
//! let mut filter = Filter::<()>::new(Rect::new(0.0, 0.0, 100.0, 100.0));
//! let blur = GaussianBlur { std_dev_x: 4.0, std_dev_y: 4.0, ..Default::default() };
//! let blurred = filter.push(Node::new(Primitive::GaussianBlur(blur), [Input::SourceAlpha]));
//! let offset = Offset { dx: 5.0, dy: 5.0 };
//! let shadow = filter.push(Node::new(Primitive::Offset(offset), [blurred]));
//! filter.push(Node::new(Primitive::Merge(Merge), [shadow, Input::SourceGraphic]));
//!
//! // The content, rendered at twice its size as premultiplied RGBA8.
//! let mut pixels = vec![0; 200 * 200 * 4];
//! let pixmap = PixmapMut::from_data(&mut pixels, 200, 200);
//! let transform = Transform::new(2.0, 2.0, 0.0, 0.0);
//! let mut backend = cpu::Backend::new(4);
//!
//! filtra::apply(&mut backend, &filter, pixmap, transform, |_, _, _, _| {});
//! ```

#![forbid(unsafe_code)]

#[cfg(feature = "cpu")]
pub mod cpu;
#[cfg(feature = "gpu")]
pub mod gpu;

pub use filtra_plan as plan;
pub use filtra_plan::{IntRect, Transform};
pub use filtra_types as types;

use filtra_plan::Plan;

/// Executes the plans of filters.
pub trait Backend {
    /// The image that filters are applied to.
    type Target<'a>;
    /// The memory that the content of an `feImage` node is rendered into.
    type Image<'a>;

    /// The pixels `target` covers.
    fn bounds(target: &Self::Target<'_>) -> IntRect;

    /// Executes `plan` on `target`, rendering each `feImage` node with `render_image`.
    fn execute(
        &mut self,
        plan: &Plan,
        target: Self::Target<'_>,
        render_image: impl FnMut(usize, IntRect, Self::Image<'_>),
    );
}

/// Applies `filter` to `target` in place, with `transform` mapping user space to its pixels.
pub fn apply<B: Backend, I>(
    backend: &mut B,
    filter: &types::Filter<I>,
    target: B::Target<'_>,
    transform: Transform,
    mut render_image: impl FnMut(&I, types::Rect, IntRect, B::Image<'_>),
) {
    let plan = filtra_plan::plan(filter, transform, B::bounds(&target));
    let images: Vec<(&I, types::Rect)> = (filter.nodes().iter())
        .zip(filter.subregions())
        .filter_map(|(node, subregion)| match &node.primitive {
            types::Primitive::Image(image) => Some((image, subregion)),
            _ => None,
        })
        .collect();

    backend.execute(&plan, target, |image, rect, data| {
        let (image, subregion) = images[image];
        render_image(image, subregion, rect, data);
    });
}
