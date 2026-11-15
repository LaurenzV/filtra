/// Thins or fattens the input (`feMorphology`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
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
pub enum MorphologyOperator {
    /// The component-wise minimum over the kernel.
    #[default]
    Erode,
    /// The component-wise maximum over the kernel.
    Dilate,
}
