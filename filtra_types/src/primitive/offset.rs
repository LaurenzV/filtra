/// Moves the input (`feOffset`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Offset {
    /// The offset along the x-axis.
    pub dx: f32,
    /// The offset along the y-axis.
    pub dy: f32,
}
