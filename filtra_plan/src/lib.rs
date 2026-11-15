//! Plans the evaluation of SVG and CSS filter effects.
//!
//! [`plan`] maps a [`Filter`](filtra_types::Filter) from user space to pixels and turns it into a
//! list of [`Step`]s on images. The steps settle everything that affects the result. A backend
//! executes them directly or as passes of its own, and [`assign_buffers`] assigns the images of
//! those passes to as few buffers as possible.
//!
//! Every step writes all pixels of its destination image. Pixels outside of a source image are
//! transparent black.

mod buffers;
mod planner;
mod rect;

pub use buffers::{Buffers, assign_buffers};
pub use filtra_types::CompositeOperator;
pub use planner::plan;
pub use rect::IntRect;

/// The steps that evaluate a filter.
#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    /// The steps in execution order.
    pub steps: Vec<Step>,
    /// The pixels of each image, indexed by [`ImageId`].
    pub rects: Vec<IntRect>,
    /// The result in sRGB.
    pub output: ImageId,
}

/// An operation that writes one image.
pub trait Pass {
    /// The images the pass reads.
    fn reads(&self) -> impl Iterator<Item = ImageId>;

    /// The image the pass writes.
    fn writes(&self) -> ImageId;
}

/// An operation that computes the result of a primitive.
#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    /// Copies the source image.
    SourceGraphic {
        /// The destination.
        dst: ImageId,
    },
    /// Copies the alpha channel of the source image with black color.
    SourceAlpha {
        /// The destination.
        dst: ImageId,
    },
    /// Converts the colors of an image.
    Convert {
        /// The source.
        src: ImageId,
        /// The destination.
        dst: ImageId,
        /// The color space of the source.
        from: ColorSpace,
        /// The color space of the destination.
        to: ColorSpace,
    },
    /// Fills with a premultiplied color.
    Flood {
        /// The destination.
        dst: ImageId,
        /// The color.
        color: [u8; 4],
    },
    /// Moves an image by whole pixels.
    Offset {
        /// The source.
        src: ImageId,
        /// The destination.
        dst: ImageId,
        /// The horizontal offset.
        dx: i64,
        /// The vertical offset.
        dy: i64,
    },
    /// Repeats the `cell` part of an image.
    Tile {
        /// The source.
        src: ImageId,
        /// The repeated pixels.
        cell: IntRect,
        /// The destination.
        dst: ImageId,
    },
    /// Composites `src` onto `backdrop`.
    Composite {
        /// The source.
        src: ImageId,
        /// The backdrop.
        backdrop: ImageId,
        /// The operator.
        operator: CompositeOperator,
        /// The destination.
        dst: ImageId,
    },
    /// Stacks images with source over, the first one at the bottom.
    Merge {
        /// The sources.
        srcs: Vec<ImageId>,
        /// The destination.
        dst: ImageId,
    },
    /// Transforms the unpremultiplied colors of an image with a matrix.
    ColorMatrix {
        /// The source.
        src: ImageId,
        /// The destination.
        dst: ImageId,
        /// A 4×5 matrix in row-major order, applied to `[R, G, B, A, 1]`.
        matrix: [f32; 20],
    },
}

impl Pass for Step {
    fn reads(&self) -> impl Iterator<Item = ImageId> {
        let (fixed, rest): ([Option<ImageId>; 2], &[ImageId]) = match self {
            Self::SourceGraphic { .. } | Self::SourceAlpha { .. } | Self::Flood { .. } => {
                ([None, None], &[])
            }
            Self::Convert { src, .. }
            | Self::Offset { src, .. }
            | Self::Tile { src, .. }
            | Self::ColorMatrix { src, .. } => ([Some(*src), None], &[]),
            Self::Composite { src, backdrop, .. } => ([Some(*src), Some(*backdrop)], &[]),
            Self::Merge { srcs, .. } => ([None, None], srcs),
        };

        fixed.into_iter().flatten().chain(rest.iter().copied())
    }

    fn writes(&self) -> ImageId {
        match self {
            Self::SourceGraphic { dst }
            | Self::SourceAlpha { dst }
            | Self::Convert { dst, .. }
            | Self::Flood { dst, .. }
            | Self::Offset { dst, .. }
            | Self::Tile { dst, .. }
            | Self::Composite { dst, .. }
            | Self::Merge { dst, .. }
            | Self::ColorMatrix { dst, .. } => *dst,
        }
    }
}

/// An image that a step or pass reads or writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ImageId(pub usize);

/// The color space of an image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorSpace {
    /// Gamma-encoded sRGB.
    Srgb,
    /// Linear-light sRGB.
    LinearRgb,
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
