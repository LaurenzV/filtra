use crate::IntRect;

/// Perlin turbulence (`feTurbulence`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Turbulence {
    /// The base frequencies along the x- and y-axis, in cycles per pixel.
    pub base_frequency: [f32; 2],
    /// The number of octaves.
    pub octaves: u32,
    /// The seed of the random numbers.
    pub seed: i64,
    /// The pixels the noise tiles, if it is stitched.
    pub stitch_tile: Option<IntRect>,
    /// Whether to sum the octaves (`fractalNoise`) instead of their absolute values.
    pub fractal_noise: bool,
    /// The position of the noise's origin in pixels.
    pub origin: [f32; 2],
}
