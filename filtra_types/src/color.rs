/// A color in gamma-encoded sRGB with unpremultiplied alpha.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Color {
    /// The red component.
    pub r: f32,
    /// The green component.
    pub g: f32,
    /// The blue component.
    pub b: f32,
    /// The alpha component.
    pub a: f32,
}

impl Color {
    /// Opaque black.
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);
    /// Opaque white.
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    /// Fully transparent black.
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    /// Creates a color from its components.
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// The color with its alpha multiplied by `opacity`.
    pub fn with_opacity(self, opacity: f32) -> Self {
        Self {
            a: self.a.clamp(0.0, 1.0) * opacity.clamp(0.0, 1.0),
            ..self
        }
    }
}

/// The color space a filter primitive operates in (`color-interpolation-filters`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ColorInterpolation {
    /// Lets the implementation choose.
    Auto,
    /// Gamma-encoded sRGB.
    Srgb,
    /// Linear-light sRGB.
    #[default]
    LinearRgb,
}

impl ColorInterpolation {
    /// The value of the `color-interpolation-filters` property.
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Srgb => "sRGB",
            Self::LinearRgb => "linearRGB",
        }
    }
}
