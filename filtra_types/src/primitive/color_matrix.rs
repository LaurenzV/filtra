/// Transforms the unpremultiplied color of each pixel with a matrix (`feColorMatrix`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorMatrix {
    /// A 5×4 matrix in row-major order, applied to `[R, G, B, A, 1]`.
    Matrix([f32; 20]),
    /// A saturation adjustment, where 0 is fully desaturated and 1 is unchanged.
    Saturate(f32),
    /// A hue rotation in degrees.
    HueRotate(f32),
    /// Converts the luminance to alpha and sets the color to black.
    LuminanceToAlpha,
}

impl ColorMatrix {
    /// The identity matrix.
    #[rustfmt::skip]
    pub const IDENTITY: [f32; 20] = [
        1.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 1.0, 0.0,
    ];
}

impl Default for ColorMatrix {
    fn default() -> Self {
        Self::Matrix(Self::IDENTITY)
    }
}
