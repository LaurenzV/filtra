/// Blends the first input (the source) onto the second (the backdrop) (`feBlend`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Blend {
    /// The blend mode.
    pub mode: BlendMode,
    /// The `no-composite` attribute.
    pub no_composite: bool,
}

/// A blend mode from Compositing and Blending Level 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

impl BlendMode {
    /// The keyword of the mode in the `mode` attribute and in CSS.
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Multiply => "multiply",
            Self::Screen => "screen",
            Self::Overlay => "overlay",
            Self::Darken => "darken",
            Self::Lighten => "lighten",
            Self::ColorDodge => "color-dodge",
            Self::ColorBurn => "color-burn",
            Self::HardLight => "hard-light",
            Self::SoftLight => "soft-light",
            Self::Difference => "difference",
            Self::Exclusion => "exclusion",
            Self::Hue => "hue",
            Self::Saturation => "saturation",
            Self::Color => "color",
            Self::Luminosity => "luminosity",
        }
    }
}
