use alloc::vec::Vec;

use super::EdgeMode;

/// Combines each pixel with its neighbors using a convolution kernel (`feConvolveMatrix`).
#[derive(Debug, Clone, PartialEq)]
pub struct ConvolveMatrix {
    /// The number of kernel columns.
    pub order_x: u32,
    /// The number of kernel rows.
    pub order_y: u32,
    /// The kernel values in row-major order.
    pub kernel: Vec<f32>,
    /// The divisor. `None` means the sum of the kernel values.
    pub divisor: Option<f32>,
    /// The value added to each component after dividing.
    pub bias: f32,
    /// The kernel column over the target pixel. `None` means `order_x / 2`.
    pub target_x: Option<i32>,
    /// The kernel row over the target pixel. `None` means `order_y / 2`.
    pub target_y: Option<i32>,
    /// How the input is extended beyond its borders.
    pub edge_mode: EdgeMode,
    /// The distance between kernel cells as `(dx, dy)`. `None` means one pixel.
    pub kernel_unit_length: Option<(f32, f32)>,
    /// Whether alpha is left unchanged.
    pub preserve_alpha: bool,
}

impl Default for ConvolveMatrix {
    fn default() -> Self {
        Self {
            order_x: 3,
            order_y: 3,
            kernel: Vec::new(),
            divisor: None,
            bias: 0.0,
            target_x: None,
            target_y: None,
            edge_mode: EdgeMode::Duplicate,
            kernel_unit_length: None,
            preserve_alpha: false,
        }
    }
}
