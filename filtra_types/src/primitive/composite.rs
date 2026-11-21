/// Composites the first input (the source) onto the second (the destination) (`feComposite`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Composite {
    /// The compositing operator.
    pub operator: CompositeOperator,
}

/// The operator of a [`Composite`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    Arithmetic { k1: f32, k2: f32, k3: f32, k4: f32 },
}
