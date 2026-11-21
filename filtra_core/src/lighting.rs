/// Lighting of the alpha of an image as a height map (`feDiffuseLighting`, `feSpecularLighting`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lighting {
    /// The reflection model.
    pub reflection: Reflection,
    /// The height of the surface where alpha is 1, in user units.
    pub surface_scale: f32,
    /// The light source in user space.
    pub light: Light,
    /// The color of the light in the color space of the result.
    pub color: [f32; 3],
    /// The number of pixels per user unit along the x- and y-axis.
    pub scale: [f32; 2],
    /// The position of the user-space origin in pixels.
    pub origin: [f32; 2],
}

/// The reflection model of a lighting primitive.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Reflection {
    /// The diffuse term of the Phong model, which is opaque.
    Diffuse {
        /// The diffuse constant `kd`.
        constant: f32,
    },
    /// The specular term of the Blinn-Phong model, whose alpha is its largest color.
    Specular {
        /// The specular constant `ks`.
        constant: f32,
        /// The specular exponent.
        exponent: f32,
    },
}

/// The light source of a lighting primitive, in user space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Light {
    /// An infinitely distant light.
    Distant {
        /// The unit vector pointing to the light.
        direction: [f32; 3],
    },
    /// A positional light.
    Point {
        /// The position.
        position: [f32; 3],
    },
    /// A positional light projecting a cone.
    Spot {
        /// The position.
        position: [f32; 3],
        /// The unit vector the light points along, or zero if it points at itself.
        direction: [f32; 3],
        /// The exponent controlling how focused the light is.
        exponent: f32,
        /// The cosine of the cone's half-angle, if the light has a cone.
        cone: Option<f32>,
    },
}
