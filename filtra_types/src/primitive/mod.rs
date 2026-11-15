mod blend;
mod color_matrix;

pub use blend::{Blend, BlendMode};
pub use color_matrix::ColorMatrix;

/// A filter primitive.
#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    /// `feBlend`.
    Blend(Blend),
    /// `feColorMatrix`.
    ColorMatrix(ColorMatrix),
}
