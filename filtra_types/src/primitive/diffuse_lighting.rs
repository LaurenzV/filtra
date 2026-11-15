use super::LightSource;
use crate::Color;

/// Lights the input's alpha as a bump map with a diffuse term (`feDiffuseLighting`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiffuseLighting {
    /// The surface height where the input alpha is 1.
    pub surface_scale: f32,
    /// The diffuse lighting constant `kd`.
    pub diffuse_constant: f32,
    /// The distance between normal samples as `(dx, dy)`. `None` means one pixel.
    pub kernel_unit_length: Option<(f32, f32)>,
    /// The `lighting-color` property.
    pub lighting_color: Color,
    /// The light source.
    pub light_source: LightSource,
}

impl DiffuseLighting {
    /// Creates the primitive with `light_source` and initial values otherwise.
    pub fn new(light_source: LightSource) -> Self {
        Self {
            surface_scale: 1.0,
            diffuse_constant: 1.0,
            kernel_unit_length: None,
            lighting_color: Color::WHITE,
            light_source,
        }
    }
}
