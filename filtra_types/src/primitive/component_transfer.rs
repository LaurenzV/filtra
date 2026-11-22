use alloc::vec::Vec;

/// Remaps each unpremultiplied color and alpha component (`feComponentTransfer`).
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComponentTransfer {
    /// The function applied to red.
    pub red: TransferFunction,
    /// The function applied to green.
    pub green: TransferFunction,
    /// The function applied to blue.
    pub blue: TransferFunction,
    /// The function applied to alpha.
    pub alpha: TransferFunction,
}

/// A function remapping a component in `[0, 1]` (`feFuncR`, `feFuncG`, `feFuncB`, `feFuncA`).
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TransferFunction {
    /// Leaves the component unchanged.
    #[default]
    Identity,
    /// Interpolates linearly between values spread evenly over `[0, 1]`; empty means identity.
    Table(Vec<f32>),
    /// A step function with one step per value, spread evenly over `[0, 1]`.
    Discrete(Vec<f32>),
    /// `slope * C + intercept`.
    Linear {
        /// The `slope` attribute.
        slope: f32,
        /// The `intercept` attribute.
        intercept: f32,
    },
    /// `amplitude * pow(C, exponent) + offset`.
    Gamma {
        /// The `amplitude` attribute.
        amplitude: f32,
        /// The `exponent` attribute.
        exponent: f32,
        /// The `offset` attribute.
        offset: f32,
    },
}

impl TransferFunction {
    /// The value of the `type` attribute.
    pub const fn keyword(&self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::Table(_) => "table",
            Self::Discrete(_) => "discrete",
            Self::Linear { .. } => "linear",
            Self::Gamma { .. } => "gamma",
        }
    }
}
