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
pub use filtra_types::{BlendMode, ColorChannel, CompositeOperator, EdgeMode, MorphologyOperator};
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
    /// Blends `src` into `backdrop`.
    Blend {
        /// The source.
        src: ImageId,
        /// The backdrop.
        backdrop: ImageId,
        /// The blend mode.
        mode: BlendMode,
        /// Whether the result is composited onto the backdrop with source over.
        composite: bool,
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
    /// Sets each pixel to the component-wise minimum or maximum of an image over the
    /// `2 rx + 1` by `2 ry + 1` pixels centered on it.
    Morphology {
        /// The source.
        src: ImageId,
        /// The destination.
        dst: ImageId,
        /// Whether to take the minimum or the maximum.
        operator: MorphologyOperator,
        /// The horizontal radius in pixels.
        rx: u32,
        /// The vertical radius in pixels.
        ry: u32,
    },
    /// Moves the pixels of `src` by the channels of `map`.
    DisplacementMap {
        /// The source.
        src: ImageId,
        /// The displacement map.
        map: ImageId,
        /// The destination.
        dst: ImageId,
        /// The horizontal and vertical displacement scale in pixels.
        scale: [f32; 2],
        /// The channels of `map` moving pixels along x and y.
        channels: [ColorChannel; 2],
    },
    /// Fills with Perlin turbulence.
    Turbulence {
        /// The destination.
        dst: ImageId,
        /// The base frequencies along the x- and y-axis, in cycles per pixel.
        base_frequency: [f32; 2],
        /// The number of octaves.
        octaves: u32,
        /// The seed of the random numbers.
        seed: i64,
        /// The pixels of the subregion, if the noise is stitched to it.
        stitch_tile: Option<IntRect>,
        /// Whether to sum the octaves (`fractalNoise`) instead of their absolute values.
        fractal_noise: bool,
        /// The position of the noise's origin in pixels.
        origin: [f32; 2],
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
    /// Remaps the unpremultiplied channels of an image with a function per channel.
    ComponentTransfer {
        /// The source.
        src: ImageId,
        /// The destination.
        dst: ImageId,
        /// The functions of red, green, blue and alpha, as the values at the start and end of
        /// equal regions of `[0, 1]`.
        functions: [Vec<[f32; 2]>; 4],
    },
    /// Sets each pixel to the sum of the pixels of `src` around it, weighted by a matrix.
    ConvolveMatrix {
        /// The source.
        src: ImageId,
        /// The destination.
        dst: ImageId,
        /// The number of columns and rows of `weights`.
        size: [u32; 2],
        /// The weights in row-major order, each multiplying the pixel it covers.
        weights: Vec<f32>,
        /// The column and row of the weight that covers the pixel being computed.
        target: [u32; 2],
        /// The value added to alpha, and times alpha to each color.
        bias: f32,
        /// How `src` is extended beyond the edges of `dst`.
        edge_mode: EdgeMode,
        /// Whether only the unpremultiplied colors are convolved, keeping alpha.
        preserve_alpha: bool,
    },
    /// Lights the alpha of `src` as a height map, sampled at the top-left corner of each pixel.
    Lighting {
        /// The source.
        src: ImageId,
        /// The destination.
        dst: ImageId,
        /// The reflection model.
        reflection: Reflection,
        /// The height of the surface where alpha is 1, in user units.
        surface_scale: f32,
        /// The light source in user space.
        light: Light,
        /// The color of the light in the color space of the result.
        color: [f32; 3],
        /// The number of pixels per user unit along the x- and y-axis.
        scale: [f32; 2],
        /// The position of the user-space origin in pixels.
        origin: [f32; 2],
    },
    /// Blurs an image with a Gaussian, treating it as transparent beyond its edges.
    GaussianBlur {
        /// The source.
        src: ImageId,
        /// The destination.
        dst: ImageId,
        /// The standard deviations along x and y in pixels; an axis with zero isn't blurred.
        std_dev: [f32; 2],
    },
}

impl Pass for Step {
    fn reads(&self) -> impl Iterator<Item = ImageId> {
        let (fixed, rest): ([Option<ImageId>; 2], &[ImageId]) = match self {
            Self::SourceGraphic { .. }
            | Self::SourceAlpha { .. }
            | Self::Flood { .. }
            | Self::Turbulence { .. } => ([None, None], &[]),
            Self::Convert { src, .. }
            | Self::Offset { src, .. }
            | Self::Tile { src, .. }
            | Self::Morphology { src, .. }
            | Self::ColorMatrix { src, .. }
            | Self::ComponentTransfer { src, .. }
            | Self::ConvolveMatrix { src, .. }
            | Self::Lighting { src, .. }
            | Self::GaussianBlur { src, .. } => ([Some(*src), None], &[]),
            Self::Composite { src, backdrop, .. } | Self::Blend { src, backdrop, .. } => {
                ([Some(*src), Some(*backdrop)], &[])
            }
            Self::DisplacementMap { src, map, .. } => ([Some(*src), Some(*map)], &[]),
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
            | Self::Blend { dst, .. }
            | Self::Merge { dst, .. }
            | Self::Morphology { dst, .. }
            | Self::DisplacementMap { dst, .. }
            | Self::Turbulence { dst, .. }
            | Self::ColorMatrix { dst, .. }
            | Self::ComponentTransfer { dst, .. }
            | Self::ConvolveMatrix { dst, .. }
            | Self::Lighting { dst, .. }
            | Self::GaussianBlur { dst, .. } => *dst,
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

/// The reflection model of [`Step::Lighting`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Reflection {
    /// The diffuse term of the Phong model, which is opaque.
    Diffuse {
        /// The diffuse constant `kd`.
        constant: f32,
    },
    /// The specular term of the Blinn-Phong model, whose alpha is its largest color.
    Specular {
        /// The specular constant `ks`.
        constant: f32,
        /// The specular exponent.
        exponent: f32,
    },
}

/// The light source of [`Step::Lighting`], in user space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Light {
    /// An infinitely distant light.
    Distant {
        /// The unit vector pointing to the light.
        direction: [f32; 3],
    },
    /// A positional light.
    Point {
        /// The position.
        position: [f32; 3],
    },
    /// A positional light projecting a cone.
    Spot {
        /// The position.
        position: [f32; 3],
        /// The unit vector the light points along, or zero if it points at itself.
        direction: [f32; 3],
        /// The exponent controlling how focused the light is.
        exponent: f32,
        /// The cosine of the cone's half-angle, if the light has a cone.
        cone: Option<f32>,
    },
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
