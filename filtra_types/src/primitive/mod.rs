#[cfg(feature = "blend")]
mod blend;
#[cfg(feature = "color-matrix")]
mod color_matrix;
#[cfg(feature = "component-transfer")]
mod component_transfer;
#[cfg(feature = "composite")]
mod composite;
#[cfg(feature = "convolve-matrix")]
mod convolve_matrix;
#[cfg(feature = "diffuse-lighting")]
mod diffuse_lighting;
#[cfg(feature = "displacement-map")]
mod displacement_map;
#[cfg(feature = "drop-shadow")]
mod drop_shadow;
#[cfg(any(feature = "gaussian-blur", feature = "convolve-matrix"))]
mod edge_mode;
#[cfg(feature = "flood")]
mod flood;
#[cfg(feature = "gaussian-blur")]
mod gaussian_blur;
#[cfg(any(feature = "diffuse-lighting", feature = "specular-lighting"))]
mod light_source;
#[cfg(feature = "merge")]
mod merge;
#[cfg(feature = "morphology")]
mod morphology;
#[cfg(feature = "offset")]
mod offset;
#[cfg(feature = "specular-lighting")]
mod specular_lighting;
#[cfg(feature = "tile")]
mod tile;
#[cfg(feature = "turbulence")]
mod turbulence;

#[cfg(feature = "blend")]
pub use blend::{Blend, BlendMode};
#[cfg(feature = "color-matrix")]
pub use color_matrix::ColorMatrix;
#[cfg(feature = "component-transfer")]
pub use component_transfer::{ComponentTransfer, TransferFunction};
#[cfg(feature = "composite")]
pub use composite::{Composite, CompositeOperator};
#[cfg(feature = "convolve-matrix")]
pub use convolve_matrix::ConvolveMatrix;
#[cfg(feature = "diffuse-lighting")]
pub use diffuse_lighting::DiffuseLighting;
#[cfg(feature = "displacement-map")]
pub use displacement_map::{ColorChannel, DisplacementMap};
#[cfg(feature = "drop-shadow")]
pub use drop_shadow::DropShadow;
#[cfg(any(feature = "gaussian-blur", feature = "convolve-matrix"))]
pub use edge_mode::EdgeMode;
#[cfg(feature = "flood")]
pub use flood::Flood;
#[cfg(feature = "gaussian-blur")]
pub use gaussian_blur::GaussianBlur;
#[cfg(any(feature = "diffuse-lighting", feature = "specular-lighting"))]
pub use light_source::{DistantLight, LightSource, PointLight, SpotLight};
#[cfg(feature = "merge")]
pub use merge::Merge;
#[cfg(feature = "morphology")]
pub use morphology::{Morphology, MorphologyOperator};
#[cfg(feature = "offset")]
pub use offset::Offset;
#[cfg(feature = "specular-lighting")]
pub use specular_lighting::SpecularLighting;
#[cfg(feature = "tile")]
pub use tile::Tile;
#[cfg(feature = "turbulence")]
pub use turbulence::{Turbulence, TurbulenceType};

/// A filter primitive.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Primitive<I> {
    /// `feBlend`.
    #[cfg(feature = "blend")]
    Blend(Blend),
    /// `feColorMatrix`.
    #[cfg(feature = "color-matrix")]
    ColorMatrix(ColorMatrix),
    /// `feComponentTransfer`.
    #[cfg(feature = "component-transfer")]
    ComponentTransfer(ComponentTransfer),
    /// `feComposite`.
    #[cfg(feature = "composite")]
    Composite(Composite),
    /// `feConvolveMatrix`.
    #[cfg(feature = "convolve-matrix")]
    ConvolveMatrix(ConvolveMatrix),
    /// `feDiffuseLighting`.
    #[cfg(feature = "diffuse-lighting")]
    DiffuseLighting(DiffuseLighting),
    /// `feDisplacementMap`.
    #[cfg(feature = "displacement-map")]
    DisplacementMap(DisplacementMap),
    /// `feDropShadow`.
    #[cfg(feature = "drop-shadow")]
    DropShadow(DropShadow),
    /// `feFlood`.
    #[cfg(feature = "flood")]
    Flood(Flood),
    /// `feGaussianBlur`.
    #[cfg(feature = "gaussian-blur")]
    GaussianBlur(GaussianBlur),
    /// `feImage`.
    Image(I),
    /// `feMerge`.
    #[cfg(feature = "merge")]
    Merge(Merge),
    /// `feMorphology`.
    #[cfg(feature = "morphology")]
    Morphology(Morphology),
    /// `feOffset`.
    #[cfg(feature = "offset")]
    Offset(Offset),
    /// `feSpecularLighting`.
    #[cfg(feature = "specular-lighting")]
    SpecularLighting(SpecularLighting),
    /// `feTile`.
    #[cfg(feature = "tile")]
    Tile(Tile),
    /// `feTurbulence`.
    #[cfg(feature = "turbulence")]
    Turbulence(Turbulence),
}
