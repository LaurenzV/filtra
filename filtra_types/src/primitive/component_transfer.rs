use alloc::vec::Vec;

/// Remaps each unpremultiplied color and alpha component (`feComponentTransfer`).
#[derive(Debug, Clone, PartialEq, Default)]
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
        ///.
        slope: f32,
        ///.
        intercept: f32,
    },
    /// `amplitude * pow(C, exponent) + offset`.
    Gamma {
        ///.
        amplitude: f32,
        ///.
        exponent: f32,
        ///.
        offset: f32,
    },
}
