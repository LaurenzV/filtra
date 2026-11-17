/// Moves the pixels of the first input by channel values of the second (`feDisplacementMap`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DisplacementMap {
    /// The displacement scale.
    // TODO: A scale per axis, since with `primitiveUnits="objectBoundingBox"` the scale is in a
    // coordinate system that stretches x by the box's width and y by its height (§9.11).
    pub scale: f32,
    /// The channel of the second input moving pixels along x.
    pub x_channel: ColorChannel,
    /// The channel of the second input moving pixels along y.
    pub y_channel: ColorChannel,
}

/// A color or alpha channel of an image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ColorChannel {
    /// Red.
    R,
    /// Green.
    G,
    /// Blue.
    B,
    /// Alpha.
    #[default]
    A,
}
