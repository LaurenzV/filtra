use filtra_types::EdgeMode;

/// A convolution with a matrix of weights (`feConvolveMatrix`).
#[derive(Debug, Clone, PartialEq)]
pub struct Convolution {
    /// The number of columns and rows of `weights`.
    pub size: [u32; 2],
    /// The weights in row-major order, each multiplying the pixel it covers.
    pub weights: Vec<f32>,
    /// The column and row of the weight that covers the pixel being computed.
    pub target: [u32; 2],
    /// The value added to alpha, and times alpha to each color.
    pub bias: f32,
    /// How the input is extended beyond the edges of the target.
    pub edge_mode: EdgeMode,
    /// Whether only the unpremultiplied colors are convolved, keeping alpha.
    pub preserve_alpha: bool,
}
