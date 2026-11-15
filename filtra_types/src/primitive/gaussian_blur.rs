use super::EdgeMode;

/// Blurs the input with a Gaussian kernel (`feGaussianBlur`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GaussianBlur {
    /// The standard deviation along the x-axis.
    pub std_dev_x: f32,
    /// The standard deviation along the y-axis.
    pub std_dev_y: f32,
    /// How the input is extended beyond its borders.
    pub edge_mode: EdgeMode,
}

impl Default for GaussianBlur {
    fn default() -> Self {
        Self {
            std_dev_x: 0.0,
            std_dev_y: 0.0,
            edge_mode: EdgeMode::None,
        }
    }
}
