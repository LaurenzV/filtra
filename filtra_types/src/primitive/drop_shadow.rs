use crate::Color;

/// Draws a blurred, offset and colored copy of the input's alpha beneath it (`feDropShadow`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DropShadow {
    /// The offset along the x-axis.
    pub dx: f32,
    /// The offset along the y-axis.
    pub dy: f32,
    /// The blur's standard deviation along the x-axis.
    pub std_dev_x: f32,
    /// The blur's standard deviation along the y-axis.
    pub std_dev_y: f32,
    /// The `flood-color` property.
    pub color: Color,
    /// The `flood-opacity` property.
    pub opacity: f32,
}

impl Default for DropShadow {
    fn default() -> Self {
        Self {
            dx: 2.0,
            dy: 2.0,
            std_dev_x: 2.0,
            std_dev_y: 2.0,
            color: Color::BLACK,
            opacity: 1.0,
        }
    }
}
