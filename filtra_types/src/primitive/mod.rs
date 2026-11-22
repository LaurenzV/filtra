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
mod turbulence;

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
pub use turbulence::{Turbulence, TurbulenceType};

/// A filter primitive.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Primitive<I> {
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
    /// `feImage`, with the content it renders.
    Image(I),
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
    /// `feTurbulence`.
    Turbulence(Turbulence),
}

impl<I> Primitive<I> {
    /// Converts from `&Primitive<I>` to `Primitive<&I>`.
    pub fn as_ref(&self) -> Primitive<&I> {
        match self {
            Self::Blend(blend) => Primitive::Blend(*blend),
            Self::ColorMatrix(matrix) => Primitive::ColorMatrix(*matrix),
            Self::ComponentTransfer(transfer) => Primitive::ComponentTransfer(transfer.clone()),
            Self::Composite(composite) => Primitive::Composite(*composite),
            Self::ConvolveMatrix(convolve) => Primitive::ConvolveMatrix(convolve.clone()),
            Self::DiffuseLighting(lighting) => Primitive::DiffuseLighting(*lighting),
            Self::DisplacementMap(map) => Primitive::DisplacementMap(*map),
            Self::DropShadow(shadow) => Primitive::DropShadow(*shadow),
            Self::Flood(flood) => Primitive::Flood(*flood),
            Self::GaussianBlur(blur) => Primitive::GaussianBlur(*blur),
            Self::Image(image) => Primitive::Image(image),
            Self::Merge(merge) => Primitive::Merge(*merge),
            Self::Morphology(morphology) => Primitive::Morphology(*morphology),
            Self::Offset(offset) => Primitive::Offset(*offset),
            Self::SpecularLighting(lighting) => Primitive::SpecularLighting(*lighting),
            Self::Tile(tile) => Primitive::Tile(*tile),
            Self::Turbulence(turbulence) => Primitive::Turbulence(*turbulence),
        }
    }

    /// The primitive with the content of an `feImage` mapped by `f`.
    pub fn try_map_image<J, E>(self, f: impl FnOnce(I) -> Result<J, E>) -> Result<Primitive<J>, E> {
        Ok(match self {
            Self::Blend(blend) => Primitive::Blend(blend),
            Self::ColorMatrix(matrix) => Primitive::ColorMatrix(matrix),
            Self::ComponentTransfer(transfer) => Primitive::ComponentTransfer(transfer),
            Self::Composite(composite) => Primitive::Composite(composite),
            Self::ConvolveMatrix(convolve) => Primitive::ConvolveMatrix(convolve),
            Self::DiffuseLighting(lighting) => Primitive::DiffuseLighting(lighting),
            Self::DisplacementMap(map) => Primitive::DisplacementMap(map),
            Self::DropShadow(shadow) => Primitive::DropShadow(shadow),
            Self::Flood(flood) => Primitive::Flood(flood),
            Self::GaussianBlur(blur) => Primitive::GaussianBlur(blur),
            Self::Image(image) => Primitive::Image(f(image)?),
            Self::Merge(merge) => Primitive::Merge(merge),
            Self::Morphology(morphology) => Primitive::Morphology(morphology),
            Self::Offset(offset) => Primitive::Offset(offset),
            Self::SpecularLighting(lighting) => Primitive::SpecularLighting(lighting),
            Self::Tile(tile) => Primitive::Tile(tile),
            Self::Turbulence(turbulence) => Primitive::Turbulence(turbulence),
        })
    }
}
