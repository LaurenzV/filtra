use crate::Color;

/// Fills the filter primitive subregion with a color (`feFlood`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Flood {
    /// The `flood-color` property.
    pub color: Color,
    /// The `flood-opacity` property.
    pub opacity: f32,
}

impl Default for Flood {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            opacity: 1.0,
        }
    }
}
