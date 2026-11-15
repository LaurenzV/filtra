mod blend;
mod color_matrix;
mod component_transfer;
mod composite;
mod convolve_matrix;
mod diffuse_lighting;
mod displacement_map;
mod edge_mode;
mod light_source;

pub use blend::{Blend, BlendMode};
pub use color_matrix::ColorMatrix;
pub use component_transfer::{ComponentTransfer, TransferFunction};
pub use composite::{Composite, CompositeOperator};
pub use convolve_matrix::ConvolveMatrix;
pub use diffuse_lighting::DiffuseLighting;
pub use displacement_map::{ColorChannel, DisplacementMap};
pub use edge_mode::EdgeMode;
pub use light_source::{DistantLight, LightSource, PointLight, SpotLight};

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
}
