mod blend;

pub use blend::{Blend, BlendMode};

/// A filter primitive.
#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    /// `feBlend`.
    Blend(Blend),
}
