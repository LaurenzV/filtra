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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    /// `feImage`, with the content it renders.
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

impl<I> Primitive<I> {
    /// Converts from `&Primitive<I>` to `Primitive<&I>`.
    pub fn as_ref(&self) -> Primitive<&I> {
        match self {
            #[cfg(feature = "blend")]
            Self::Blend(blend) => Primitive::Blend(*blend),
            #[cfg(feature = "color-matrix")]
            Self::ColorMatrix(matrix) => Primitive::ColorMatrix(*matrix),
            #[cfg(feature = "component-transfer")]
            Self::ComponentTransfer(transfer) => Primitive::ComponentTransfer(transfer.clone()),
            #[cfg(feature = "composite")]
            Self::Composite(composite) => Primitive::Composite(*composite),
            #[cfg(feature = "convolve-matrix")]
            Self::ConvolveMatrix(convolve) => Primitive::ConvolveMatrix(convolve.clone()),
            #[cfg(feature = "diffuse-lighting")]
            Self::DiffuseLighting(lighting) => Primitive::DiffuseLighting(*lighting),
            #[cfg(feature = "displacement-map")]
            Self::DisplacementMap(map) => Primitive::DisplacementMap(*map),
            #[cfg(feature = "drop-shadow")]
            Self::DropShadow(shadow) => Primitive::DropShadow(*shadow),
            #[cfg(feature = "flood")]
            Self::Flood(flood) => Primitive::Flood(*flood),
            #[cfg(feature = "gaussian-blur")]
            Self::GaussianBlur(blur) => Primitive::GaussianBlur(*blur),
            Self::Image(image) => Primitive::Image(image),
            #[cfg(feature = "merge")]
            Self::Merge(merge) => Primitive::Merge(*merge),
            #[cfg(feature = "morphology")]
            Self::Morphology(morphology) => Primitive::Morphology(*morphology),
            #[cfg(feature = "offset")]
            Self::Offset(offset) => Primitive::Offset(*offset),
            #[cfg(feature = "specular-lighting")]
            Self::SpecularLighting(lighting) => Primitive::SpecularLighting(*lighting),
            #[cfg(feature = "tile")]
            Self::Tile(tile) => Primitive::Tile(*tile),
            #[cfg(feature = "turbulence")]
            Self::Turbulence(turbulence) => Primitive::Turbulence(*turbulence),
        }
    }

    /// The primitive with the content of an `feImage` mapped by `f`.
    pub fn try_map_image<J, E>(self, f: impl FnOnce(I) -> Result<J, E>) -> Result<Primitive<J>, E> {
        Ok(match self {
            #[cfg(feature = "blend")]
            Self::Blend(blend) => Primitive::Blend(blend),
            #[cfg(feature = "color-matrix")]
            Self::ColorMatrix(matrix) => Primitive::ColorMatrix(matrix),
            #[cfg(feature = "component-transfer")]
            Self::ComponentTransfer(transfer) => Primitive::ComponentTransfer(transfer),
            #[cfg(feature = "composite")]
            Self::Composite(composite) => Primitive::Composite(composite),
            #[cfg(feature = "convolve-matrix")]
            Self::ConvolveMatrix(convolve) => Primitive::ConvolveMatrix(convolve),
            #[cfg(feature = "diffuse-lighting")]
            Self::DiffuseLighting(lighting) => Primitive::DiffuseLighting(lighting),
            #[cfg(feature = "displacement-map")]
            Self::DisplacementMap(map) => Primitive::DisplacementMap(map),
            #[cfg(feature = "drop-shadow")]
            Self::DropShadow(shadow) => Primitive::DropShadow(shadow),
            #[cfg(feature = "flood")]
            Self::Flood(flood) => Primitive::Flood(flood),
            #[cfg(feature = "gaussian-blur")]
            Self::GaussianBlur(blur) => Primitive::GaussianBlur(blur),
            Self::Image(image) => Primitive::Image(f(image)?),
            #[cfg(feature = "merge")]
            Self::Merge(merge) => Primitive::Merge(merge),
            #[cfg(feature = "morphology")]
            Self::Morphology(morphology) => Primitive::Morphology(morphology),
            #[cfg(feature = "offset")]
            Self::Offset(offset) => Primitive::Offset(offset),
            #[cfg(feature = "specular-lighting")]
            Self::SpecularLighting(lighting) => Primitive::SpecularLighting(lighting),
            #[cfg(feature = "tile")]
            Self::Tile(tile) => Primitive::Tile(tile),
            #[cfg(feature = "turbulence")]
            Self::Turbulence(turbulence) => Primitive::Turbulence(turbulence),
        })
    }
}
