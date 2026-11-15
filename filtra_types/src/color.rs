/// A color in gamma-encoded sRGB with unpremultiplied alpha.
#[derive(Debug, Clone, Copy, PartialEq)]
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

    /// The color as premultiplied RGBA8.
    pub fn to_premultiplied_rgba8(self) -> [u8; 4] {
        let alpha = self.a.clamp(0.0, 1.0);
        let to_u8 = |value: f32| (value * 255.0 + 0.5) as u8;

        [
            to_u8(self.r.clamp(0.0, 1.0) * alpha),
            to_u8(self.g.clamp(0.0, 1.0) * alpha),
            to_u8(self.b.clamp(0.0, 1.0) * alpha),
            to_u8(alpha),
        ]
    }
}

/// The color space a filter primitive operates in (`color-interpolation-filters`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ColorInterpolation {
    /// Lets the implementation choose.
    Auto,
    /// Gamma-encoded sRGB.
    Srgb,
    /// Linear-light sRGB.
    #[default]
    LinearRgb,
}
