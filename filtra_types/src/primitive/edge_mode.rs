/// How an input is extended beyond its borders (`edgeMode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EdgeMode {
    /// Repeat the color values at the nearest edge.
    Duplicate,
    /// Take the color values from the opposite edge.
    Wrap,
    /// Take the color values mirrored across the edge.
    Mirror,
    /// Extend with transparent black.
    None,
}

impl EdgeMode {
    /// The value of the `edgeMode` attribute.
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Duplicate => "duplicate",
            Self::Wrap => "wrap",
            Self::Mirror => "mirror",
            Self::None => "none",
        }
    }
}
