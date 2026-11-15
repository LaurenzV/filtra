mod blend;
mod color_matrix;
mod component_transfer;
mod composite;
mod convolve_matrix;
mod edge_mode;

pub use blend::{Blend, BlendMode};
pub use color_matrix::ColorMatrix;
pub use component_transfer::{ComponentTransfer, TransferFunction};
pub use composite::{Composite, CompositeOperator};
pub use convolve_matrix::ConvolveMatrix;
pub use edge_mode::EdgeMode;

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
}
