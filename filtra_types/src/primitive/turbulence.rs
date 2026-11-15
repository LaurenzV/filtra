/// Generates Perlin turbulence (`feTurbulence`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Turbulence {
    /// The base frequency along the x-axis.
    pub base_frequency_x: f32,
    /// The base frequency along the y-axis.
    pub base_frequency_y: f32,
    /// The number of octaves.
    pub num_octaves: i32,
    /// The random seed, truncated towards zero.
    pub seed: f32,
    /// Whether the noise tiles seamlessly across the subregion.
    pub stitch_tiles: bool,
    /// The noise function.
    pub kind: TurbulenceType,
}

impl Default for Turbulence {
    fn default() -> Self {
        Self {
            base_frequency_x: 0.0,
            base_frequency_y: 0.0,
            num_octaves: 1,
            seed: 0.0,
            stitch_tiles: false,
            kind: TurbulenceType::Turbulence,
        }
    }
}

/// The noise function of a [`Turbulence`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TurbulenceType {
    /// The sum of the noise octaves.
    FractalNoise,
    /// The sum of the absolute values of the noise octaves.
    #[default]
    Turbulence,
}
