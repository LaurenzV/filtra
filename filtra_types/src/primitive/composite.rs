/// Composites the first input (the source) onto the second (the destination) (`feComposite`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Composite {
    /// The compositing operator.
    pub operator: CompositeOperator,
}

/// The operator of a [`Composite`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CompositeOperator {
    /// Porter-Duff source over.
    #[default]
    Over,
    /// Porter-Duff source in.
    In,
    /// Porter-Duff source out.
    Out,
    /// Porter-Duff source atop.
    Atop,
    /// Porter-Duff XOR.
    Xor,
    /// Porter-Duff lighter (plus).
    Lighter,
    /// `k1 * i1 * i2 + k2 * i1 + k3 * i2 + k4` per channel.
    Arithmetic {
        /// The `k1` attribute.
        k1: f32,
        /// The `k2` attribute.
        k2: f32,
        /// The `k3` attribute.
        k3: f32,
        /// The `k4` attribute.
        k4: f32,
    },
}
