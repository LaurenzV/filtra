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
