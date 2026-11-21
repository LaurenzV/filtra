/// The color space of an image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorSpace {
    /// Gamma-encoded sRGB.
    Srgb,
    /// Linear-light sRGB.
    LinearRgb,
}
