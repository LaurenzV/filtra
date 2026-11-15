mod blend;
mod color_matrix;
mod component_transfer;
mod composite;
mod convolve_matrix;
mod diffuse_lighting;
mod displacement_map;
mod drop_shadow;
mod edge_mode;
mod flood;
mod gaussian_blur;
mod light_source;
mod merge;
mod morphology;
mod offset;
mod specular_lighting;
mod tile;

pub use blend::{Blend, BlendMode};
pub use color_matrix::ColorMatrix;
pub use component_transfer::{ComponentTransfer, TransferFunction};
pub use composite::{Composite, CompositeOperator};
pub use convolve_matrix::ConvolveMatrix;
pub use diffuse_lighting::DiffuseLighting;
pub use displacement_map::{ColorChannel, DisplacementMap};
pub use drop_shadow::DropShadow;
pub use edge_mode::EdgeMode;
pub use flood::Flood;
pub use gaussian_blur::GaussianBlur;
pub use light_source::{DistantLight, LightSource, PointLight, SpotLight};
pub use merge::Merge;
pub use morphology::{Morphology, MorphologyOperator};
pub use offset::Offset;
pub use specular_lighting::SpecularLighting;
pub use tile::Tile;

/// A filter primitive.
#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    /// `feBlend`.
    Blend(Blend),
    /// `feColorMatrix`.
    ColorMatrix(ColorMatrix),
    /// `feComponentTransfer`.
    ComponentTransfer(ComponentTransfer),
    /// `feComposite`.
    Composite(Composite),
    /// `feConvolveMatrix`.
    ConvolveMatrix(ConvolveMatrix),
    /// `feDiffuseLighting`.
    DiffuseLighting(DiffuseLighting),
    /// `feDisplacementMap`.
    DisplacementMap(DisplacementMap),
    /// `feDropShadow`.
    DropShadow(DropShadow),
    /// `feFlood`.
    Flood(Flood),
    /// `feGaussianBlur`.
    GaussianBlur(GaussianBlur),
    /// `feMerge`.
    Merge(Merge),
    /// `feMorphology`.
    Morphology(Morphology),
    /// `feOffset`.
    Offset(Offset),
    /// `feSpecularLighting`.
    SpecularLighting(SpecularLighting),
    /// `feTile`.
    Tile(Tile),
}
