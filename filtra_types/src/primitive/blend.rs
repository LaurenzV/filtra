/// Blends the first input (the source) onto the second (the backdrop) (`feBlend`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Blend {
    /// The blend mode.
    pub mode: BlendMode,
    /// The `no-composite` attribute.
    pub no_composite: bool,
}

/// A blend mode from Compositing and Blending Level 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BlendMode {
    /// `normal`.
    #[default]
    Normal,
    /// `multiply`.
    Multiply,
    /// `screen`.
    Screen,
    /// `overlay`.
    Overlay,
    /// `darken`.
    Darken,
    /// `lighten`.
    Lighten,
    /// `color-dodge`.
    ColorDodge,
    /// `color-burn`.
    ColorBurn,
    /// `hard-light`.
    HardLight,
    /// `soft-light`.
    SoftLight,
    /// `difference`.
    Difference,
    /// `exclusion`.
    Exclusion,
    /// `hue`.
    Hue,
    /// `saturation`.
    Saturation,
    /// `color`.
    Color,
    /// `luminosity`.
    Luminosity,
}
