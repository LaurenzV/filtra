//! Turns a filter into steps on pixel images, which settle everything that affects the result.
//!
//! Every step writes all pixels of its destination image. Pixels outside of a source image are
//! transparent black.

mod buffers;
mod planner;

pub(crate) use buffers::assign_buffers;
pub(crate) use planner::plan;

#[cfg(feature = "convolve-matrix")]
use filtra_core::Convolution;
#[cfg(any(feature = "diffuse-lighting", feature = "specular-lighting"))]
use filtra_core::Lighting;
#[cfg(feature = "turbulence")]
use filtra_core::Turbulence;
use filtra_core::{ColorSpace, IntRect};
#[cfg(feature = "blend")]
use filtra_types::BlendMode;
#[cfg(feature = "displacement-map")]
use filtra_types::ColorChannel;
#[cfg(feature = "composite")]
use filtra_types::CompositeOperator;
#[cfg(feature = "morphology")]
use filtra_types::MorphologyOperator;

/// The steps that evaluate a filter.
#[derive(Debug)]
pub(crate) struct Plan {
    /// The steps in execution order.
    pub(crate) steps: Vec<Step>,
    /// The pixels of each image, indexed by [`ImageId`].
    pub(crate) rects: Vec<IntRect>,
    /// The result in sRGB.
    pub(crate) output: ImageId,
}

/// An operation that computes the result of a primitive.
#[derive(Debug)]
pub(crate) enum Step {
    /// Copies the alpha channel of an image with black color.
    Alpha { src: ImageId, dst: ImageId },
    /// Converts the colors of an image.
    Convert {
        src: ImageId,
        dst: ImageId,
        from: ColorSpace,
        to: ColorSpace,
    },
    /// Fills with a premultiplied color.
    Flood { dst: ImageId, color: [u8; 4] },
    /// Fills with the image of an `feImage` node, which the caller renders.
    Image {
        dst: ImageId,
        /// The index of the `feImage` node in the filter.
        node: usize,
    },
    #[cfg(any(
        feature = "offset",
        feature = "morphology",
        feature = "convolve-matrix"
    ))]
    /// Moves an image by whole pixels.
    Offset {
        src: ImageId,
        dst: ImageId,
        dx: i64,
        dy: i64,
    },
    #[cfg(feature = "tile")]
    /// Repeats the `cell` part of an image.
    Tile {
        src: ImageId,
        /// The repeated pixels.
        cell: IntRect,
        dst: ImageId,
    },
    #[cfg(feature = "composite")]
    /// Composites `src` onto `backdrop`.
    Composite {
        src: ImageId,
        backdrop: ImageId,
        operator: CompositeOperator,
        dst: ImageId,
    },
    #[cfg(feature = "blend")]
    /// Blends `src` into `backdrop`.
    Blend {
        src: ImageId,
        backdrop: ImageId,
        mode: BlendMode,
        /// Whether the result is composited onto the backdrop with source over.
        composite: bool,
        dst: ImageId,
    },
    #[cfg(feature = "merge")]
    /// Stacks images with source over, the first one at the bottom.
    Merge { srcs: Vec<ImageId>, dst: ImageId },
    #[cfg(feature = "morphology")]
    /// Sets each pixel to the component-wise minimum or maximum of an image over the
    /// `2 rx + 1` by `2 ry + 1` pixels centered on it.
    Morphology {
        src: ImageId,
        dst: ImageId,
        operator: MorphologyOperator,
        rx: u32,
        ry: u32,
    },
    #[cfg(feature = "displacement-map")]
    /// Moves the pixels of `src` by the channels of `map`.
    DisplacementMap {
        src: ImageId,
        map: ImageId,
        dst: ImageId,
        /// The horizontal and vertical displacement scale in pixels.
        scale: [f32; 2],
        /// The channels of `map` moving pixels along x and y.
        channels: [ColorChannel; 2],
    },
    #[cfg(feature = "turbulence")]
    /// Fills with Perlin turbulence.
    Turbulence {
        dst: ImageId,
        turbulence: Turbulence,
    },
    #[cfg(feature = "color-matrix")]
    /// Transforms the unpremultiplied colors of an image with a matrix.
    ColorMatrix {
        src: ImageId,
        dst: ImageId,
        /// A 4×5 matrix in row-major order, applied to `[R, G, B, A, 1]`.
        matrix: [f32; 20],
    },
    #[cfg(feature = "component-transfer")]
    /// Remaps the unpremultiplied channels of an image with a function per channel.
    ComponentTransfer {
        src: ImageId,
        dst: ImageId,
        /// The functions of red, green, blue and alpha, as the values at the start and end of
        /// equal regions of `[0, 1]`.
        functions: [Vec<[f32; 2]>; 4],
    },
    #[cfg(feature = "convolve-matrix")]
    /// Sets each pixel to the sum of the pixels of `src` around it, weighted by a matrix.
    ConvolveMatrix {
        src: ImageId,
        dst: ImageId,
        convolution: Convolution,
    },
    #[cfg(any(feature = "diffuse-lighting", feature = "specular-lighting"))]
    /// Lights the alpha of `src` as a height map, sampled at the corner of each pixel with the
    /// smallest user-space coordinates.
    Lighting {
        src: ImageId,
        dst: ImageId,
        lighting: Lighting,
    },
    #[cfg(feature = "gaussian-blur")]
    /// Blurs an image with a Gaussian.
    GaussianBlur {
        src: ImageId,
        dst: ImageId,
        /// The standard deviations along x and y in pixels.
        std_dev: [f32; 2],
    },
}

impl Step {
    pub(crate) fn reads(&self) -> impl Iterator<Item = ImageId> {
        let (fixed, rest): ([Option<ImageId>; 2], &[ImageId]) = match self {
            Self::Flood { .. } | Self::Image { .. } => ([None, None], &[]),
            Self::Alpha { src, .. } | Self::Convert { src, .. } => ([Some(*src), None], &[]),
            #[cfg(any(
                feature = "offset",
                feature = "morphology",
                feature = "convolve-matrix"
            ))]
            Self::Offset { src, .. } => ([Some(*src), None], &[]),
            #[cfg(feature = "tile")]
            Self::Tile { src, .. } => ([Some(*src), None], &[]),
            #[cfg(feature = "composite")]
            Self::Composite { src, backdrop, .. } => ([Some(*src), Some(*backdrop)], &[]),
            #[cfg(feature = "blend")]
            Self::Blend { src, backdrop, .. } => ([Some(*src), Some(*backdrop)], &[]),
            #[cfg(feature = "merge")]
            Self::Merge { srcs, .. } => ([None, None], srcs),
            #[cfg(feature = "morphology")]
            Self::Morphology { src, .. } => ([Some(*src), None], &[]),
            #[cfg(feature = "displacement-map")]
            Self::DisplacementMap { src, map, .. } => ([Some(*src), Some(*map)], &[]),
            #[cfg(feature = "turbulence")]
            Self::Turbulence { .. } => ([None, None], &[]),
            #[cfg(feature = "color-matrix")]
            Self::ColorMatrix { src, .. } => ([Some(*src), None], &[]),
            #[cfg(feature = "component-transfer")]
            Self::ComponentTransfer { src, .. } => ([Some(*src), None], &[]),
            #[cfg(feature = "convolve-matrix")]
            Self::ConvolveMatrix { src, .. } => ([Some(*src), None], &[]),
            #[cfg(any(feature = "diffuse-lighting", feature = "specular-lighting"))]
            Self::Lighting { src, .. } => ([Some(*src), None], &[]),
            #[cfg(feature = "gaussian-blur")]
            Self::GaussianBlur { src, .. } => ([Some(*src), None], &[]),
        };

        fixed.into_iter().flatten().chain(rest.iter().copied())
    }

    pub(crate) fn writes(&self) -> ImageId {
        match self {
            Self::Alpha { dst, .. }
            | Self::Convert { dst, .. }
            | Self::Flood { dst, .. }
            | Self::Image { dst, .. } => *dst,
            #[cfg(any(
                feature = "offset",
                feature = "morphology",
                feature = "convolve-matrix"
            ))]
            Self::Offset { dst, .. } => *dst,
            #[cfg(feature = "tile")]
            Self::Tile { dst, .. } => *dst,
            #[cfg(feature = "composite")]
            Self::Composite { dst, .. } => *dst,
            #[cfg(feature = "blend")]
            Self::Blend { dst, .. } => *dst,
            #[cfg(feature = "merge")]
            Self::Merge { dst, .. } => *dst,
            #[cfg(feature = "morphology")]
            Self::Morphology { dst, .. } => *dst,
            #[cfg(feature = "displacement-map")]
            Self::DisplacementMap { dst, .. } => *dst,
            #[cfg(feature = "turbulence")]
            Self::Turbulence { dst, .. } => *dst,
            #[cfg(feature = "color-matrix")]
            Self::ColorMatrix { dst, .. } => *dst,
            #[cfg(feature = "component-transfer")]
            Self::ComponentTransfer { dst, .. } => *dst,
            #[cfg(feature = "convolve-matrix")]
            Self::ConvolveMatrix { dst, .. } => *dst,
            #[cfg(any(feature = "diffuse-lighting", feature = "specular-lighting"))]
            Self::Lighting { dst, .. } => *dst,
            #[cfg(feature = "gaussian-blur")]
            Self::GaussianBlur { dst, .. } => *dst,
        }
    }
}

/// An image that a step reads or writes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ImageId(pub(crate) usize);
