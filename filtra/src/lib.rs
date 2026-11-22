//! Applies SVG and CSS filter effects to pixmaps.
//!
//! This crate applies arbitrary filter graphs, as defined in
//! [Filter Effects Module Level 1](https://drafts.fxtf.org/filter-effects-1/), on the CPU, with
//! SIMD kernels for NEON, AVX2 and WebAssembly.
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
//! let mut pool = cpu::BufferPool::new(4);
//!
//! filtra::apply(&filter, pixmap, &mut pool, transform, |_, _, _, _| {});
//! ```
//!
//! # Applying a filter
//!
//! [`apply`] filters a target in place. The target holds the `SourceGraphic`, as a
//! [`cpu::PixmapMut`] of premultiplied sRGB RGBA8. Afterwards, it holds the result, and every pixel
//! outside the filter region is transparent.
//!
//! The [`Transform`] maps the user space of the filter to the pixels of the target, and a negative
//! scale flips an axis. Rotations and skews are left to the caller: apply the filter to content
//! rendered without them, then draw the result with the rotation and skew that remain.
//!
//! For each `feImage` node the result needs, `apply` calls `render_image` with the node's content,
//! its filter primitive subregion in user space, and the pixels to render the content into: a rect
//! in the pixel space of the target, and a transparent buffer of that size in premultiplied RGBA8.
//!
//! # Limitations
//!
//! - The `BackgroundImage`, `BackgroundAlpha`, `FillPaint` and `StrokePaint` inputs are not
//!   supported.
//! - `edgeMode` on `feGaussianBlur` is ignored, as is `kernelUnitLength` on `feConvolveMatrix`,
//!   `feDiffuseLighting` and `feSpecularLighting`.
//! - On WebAssembly, the kernels only use SIMD when compiled with the `simd128` target feature, for
//!   example with `RUSTFLAGS="-C target-feature=+simd128"`.
//!
//! # Features
//!
//! Each filter primitive has a feature of the same name in kebab case, such as `gaussian-blur`,
//! which lets `apply` evaluate it. `all`, the default, enables every primitive, and `css-functions`
//! enables the ones that filter functions need. `apply` renders a primitive whose feature is
//! disabled as transparent black.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]

pub mod cpu;
mod plan;

pub use filtra_core::IntRect;
pub use filtra_types as types;

/// Applies `filter` to `target` in place, with `transform` mapping user space to its pixels.
pub fn apply<I>(
    filter: &types::Filter<I>,
    target: cpu::PixmapMut<'_>,
    pool: &mut cpu::BufferPool,
    transform: Transform,
    mut render_image: impl FnMut(&I, types::Rect, IntRect, &mut [u8]),
) {
    let plan = plan::plan(filter, transform, target.bounds());
    let subregions = filter.subregions();

    pool.execute(&plan, target, &mut |node, rect, data| {
        if let types::Primitive::Image(image) = &filter.nodes()[node].primitive {
            render_image(image, subregions[node], rect, data);
        }
    });
}

/// A scale and translation that maps user space to pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    /// The horizontal scale.
    pub sx: f32,
    /// The vertical scale.
    pub sy: f32,
    /// The horizontal translation in pixels.
    pub tx: f32,
    /// The vertical translation in pixels.
    pub ty: f32,
}

impl Transform {
    /// Creates a transform from its scales and translations.
    pub const fn new(sx: f32, sy: f32, tx: f32, ty: f32) -> Self {
        Self { sx, sy, tx, ty }
    }
}
