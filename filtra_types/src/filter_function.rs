use alloc::vec;

use crate::{
    Color, ColorMatrix, ComponentTransfer, DropShadow, EdgeMode, GaussianBlur, Primitive,
    TransferFunction,
};

/// A CSS filter function, with percentages converted so that 100% is 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterFunction {
    /// `blur()` with the standard deviation.
    Blur(f32),
    /// `brightness()` with the amount.
    Brightness(f32),
    /// `contrast()` with the amount.
    Contrast(f32),
    /// `drop-shadow()`.
    DropShadow {
        /// The shadow offset along the x-axis.
        dx: f32,
        /// The shadow offset along the y-axis.
        dy: f32,
        /// The standard deviation of the shadow blur.
        std_dev: f32,
        /// The shadow color.
        color: Color,
    },
    /// `grayscale()` with the amount, clamped to 1.
    Grayscale(f32),
    /// `hue-rotate()` with the angle in degrees.
    HueRotate(f32),
    /// `invert()` with the amount, clamped to 1.
    Invert(f32),
    /// `opacity()` with the amount, clamped to 1.
    Opacity(f32),
    /// `saturate()` with the amount.
    Saturate(f32),
    /// `sepia()` with the amount, clamped to 1.
    Sepia(f32),
}

impl FilterFunction {
    /// The equivalent filter primitive.
    pub fn to_primitive(&self) -> Primitive {
        match *self {
            Self::Blur(std_dev) => Primitive::GaussianBlur(GaussianBlur {
                std_dev_x: std_dev,
                std_dev_y: std_dev,
                edge_mode: EdgeMode::None,
            }),
            Self::Brightness(amount) => rgb_transfer(TransferFunction::Linear {
                slope: amount,
                intercept: 0.0,
            }),
            Self::Contrast(amount) => rgb_transfer(TransferFunction::Linear {
                slope: amount,
                intercept: -(0.5 * amount) + 0.5,
            }),
            Self::DropShadow {
                dx,
                dy,
                std_dev,
                color,
            } => Primitive::DropShadow(DropShadow {
                dx,
                dy,
                std_dev_x: std_dev,
                std_dev_y: std_dev,
                color,
                opacity: 1.0,
            }),
            Self::Grayscale(amount) => {
                let a = 1.0 - amount.min(1.0);
                Primitive::ColorMatrix(ColorMatrix::Matrix([
                    0.2126 + 0.7874 * a,
                    0.7152 - 0.7152 * a,
                    0.0722 - 0.0722 * a,
                    0.0,
                    0.0,
                    0.2126 - 0.2126 * a,
                    0.7152 + 0.2848 * a,
                    0.0722 - 0.0722 * a,
                    0.0,
                    0.0,
                    0.2126 - 0.2126 * a,
                    0.7152 - 0.7152 * a,
                    0.0722 + 0.9278 * a,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    1.0,
                    0.0,
                ]))
            }
            Self::HueRotate(angle) => Primitive::ColorMatrix(ColorMatrix::HueRotate(angle)),
            Self::Invert(amount) => {
                let amount = amount.min(1.0);
                rgb_transfer(TransferFunction::Table(vec![amount, 1.0 - amount]))
            }
            Self::Opacity(amount) => Primitive::ComponentTransfer(ComponentTransfer {
                alpha: TransferFunction::Table(vec![0.0, amount.min(1.0)]),
                ..ComponentTransfer::default()
            }),
            Self::Saturate(amount) => Primitive::ColorMatrix(ColorMatrix::Saturate(amount)),
            Self::Sepia(amount) => {
                let a = 1.0 - amount.min(1.0);
                Primitive::ColorMatrix(ColorMatrix::Matrix([
                    0.393 + 0.607 * a,
                    0.769 - 0.769 * a,
                    0.189 - 0.189 * a,
                    0.0,
                    0.0,
                    0.349 - 0.349 * a,
                    0.686 + 0.314 * a,
                    0.168 - 0.168 * a,
                    0.0,
                    0.0,
                    0.272 - 0.272 * a,
                    0.534 - 0.534 * a,
                    0.131 + 0.869 * a,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    1.0,
                    0.0,
                ]))
            }
        }
    }
}

fn rgb_transfer(function: TransferFunction) -> Primitive {
    Primitive::ComponentTransfer(ComponentTransfer {
        red: function.clone(),
        green: function.clone(),
        blue: function,
        alpha: TransferFunction::Identity,
    })
}
