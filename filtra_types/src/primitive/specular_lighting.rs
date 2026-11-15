use super::LightSource;
use crate::Color;

/// Lights the input's alpha as a bump map with a specular term (`feSpecularLighting`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpecularLighting {
    /// The surface height where the input alpha is 1.
    pub surface_scale: f32,
    /// The specular lighting constant `ks`.
    pub specular_constant: f32,
    /// The exponent of the specular term; larger is shinier.
    pub specular_exponent: f32,
    /// The distance between normal samples as `(dx, dy)`. `None` means one pixel.
    pub kernel_unit_length: Option<(f32, f32)>,
    /// The `lighting-color` property.
    pub lighting_color: Color,
    /// The light source.
    pub light_source: LightSource,
}

impl SpecularLighting {
    /// Creates the primitive with `light_source` and initial values otherwise.
    pub fn new(light_source: LightSource) -> Self {
        Self {
            surface_scale: 1.0,
            specular_constant: 1.0,
            specular_exponent: 1.0,
            kernel_unit_length: None,
            lighting_color: Color::WHITE,
            light_source,
        }
    }
}
