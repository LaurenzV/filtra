/// Thins or fattens the input (`feMorphology`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Morphology {
    /// Whether to erode or dilate.
    pub operator: MorphologyOperator,
    /// The kernel radius along the x-axis.
    pub radius_x: f32,
    /// The kernel radius along the y-axis.
    pub radius_y: f32,
}

/// Whether a [`Morphology`] thins or fattens the input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MorphologyOperator {
    /// The component-wise minimum over the kernel.
    #[default]
    Erode,
    /// The component-wise maximum over the kernel.
    Dilate,
}

impl MorphologyOperator {
    /// The value of the `operator` attribute.
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Erode => "erode",
            Self::Dilate => "dilate",
        }
    }
}
