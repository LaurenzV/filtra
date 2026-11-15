/// The light source of a lighting primitive.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightSource {
    /// An infinitely distant light.
    Distant(DistantLight),
    /// A point light.
    Point(PointLight),
    /// A spot light.
    Spot(SpotLight),
}

/// An infinitely distant light (`feDistantLight`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DistantLight {
    /// The direction in the XY plane in degrees, clockwise from the x-axis.
    pub azimuth: f32,
    /// The direction from the XY plane towards the z-axis in degrees.
    pub elevation: f32,
}

/// A positional light (`fePointLight`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PointLight {
    /// The x position.
    pub x: f32,
    /// The y position.
    pub y: f32,
    /// The z position, with positive z pointing towards the viewer.
    pub z: f32,
}

/// A positional light projecting a cone (`feSpotLight`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpotLight {
    /// The x position.
    pub x: f32,
    /// The y position.
    pub y: f32,
    /// The z position, with positive z pointing towards the viewer.
    pub z: f32,
    /// The x position the light points at.
    pub points_at_x: f32,
    /// The y position the light points at.
    pub points_at_y: f32,
    /// The z position the light points at.
    pub points_at_z: f32,
    /// The exponent controlling how focused the light is.
    pub specular_exponent: f32,
    /// The cone's half-angle in degrees. `None` means no cone.
    pub limiting_cone_angle: Option<f32>,
}

impl Default for SpotLight {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            points_at_x: 0.0,
            points_at_y: 0.0,
            points_at_z: 0.0,
            specular_exponent: 1.0,
            limiting_cone_angle: None,
        }
    }
}
