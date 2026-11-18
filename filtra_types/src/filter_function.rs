#[cfg(feature = "component-transfer")]
use alloc::vec;

#[cfg(feature = "color-matrix")]
use crate::ColorMatrix;
#[cfg(feature = "drop-shadow")]
use crate::{Color, DropShadow};
#[cfg(feature = "component-transfer")]
use crate::{ComponentTransfer, TransferFunction};
#[cfg(feature = "gaussian-blur")]
use crate::{EdgeMode, GaussianBlur};
use crate::{Primitive, Rect};

/// A CSS filter function, with percentages converted so that 100% is 1.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum FilterFunction {
    /// `blur()` with the standard deviation.
    #[cfg(feature = "gaussian-blur")]
    Blur(f32),
    /// `brightness()` with the amount.
    #[cfg(feature = "component-transfer")]
    Brightness(f32),
    /// `contrast()` with the amount.
    #[cfg(feature = "component-transfer")]
    Contrast(f32),
    /// `drop-shadow()`.
    #[cfg(feature = "drop-shadow")]
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
    #[cfg(feature = "color-matrix")]
    Grayscale(f32),
    /// `hue-rotate()` with the angle in degrees.
    #[cfg(feature = "color-matrix")]
    HueRotate(f32),
    /// `invert()` with the amount, clamped to 1.
    #[cfg(feature = "component-transfer")]
    Invert(f32),
    /// `opacity()` with the amount, clamped to 1.
    #[cfg(feature = "component-transfer")]
    Opacity(f32),
    /// `saturate()` with the amount.
    #[cfg(feature = "color-matrix")]
    Saturate(f32),
    /// `sepia()` with the amount, clamped to 1.
    #[cfg(feature = "color-matrix")]
    Sepia(f32),
}

impl FilterFunction {
    /// The equivalent filter primitive.
    pub fn to_primitive<I>(&self) -> Primitive<I> {
        match *self {
            // TODO: `EdgeMode::Duplicate` (§13.1) once the planner supports edge modes.
            #[cfg(feature = "gaussian-blur")]
            Self::Blur(std_dev) => Primitive::GaussianBlur(GaussianBlur {
                std_dev_x: std_dev,
                std_dev_y: std_dev,
                edge_mode: EdgeMode::None,
            }),
            #[cfg(feature = "component-transfer")]
            Self::Brightness(amount) => rgb_transfer(TransferFunction::Linear {
                slope: amount,
                intercept: 0.0,
            }),
            #[cfg(feature = "component-transfer")]
            Self::Contrast(amount) => rgb_transfer(TransferFunction::Linear {
                slope: amount,
                intercept: -(0.5 * amount) + 0.5,
            }),
            #[cfg(feature = "drop-shadow")]
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
            #[cfg(feature = "color-matrix")]
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
            #[cfg(feature = "color-matrix")]
            Self::HueRotate(angle) => Primitive::ColorMatrix(ColorMatrix::HueRotate(angle)),
            #[cfg(feature = "component-transfer")]
            Self::Invert(amount) => {
                let amount = amount.min(1.0);
                rgb_transfer(TransferFunction::Table(vec![amount, 1.0 - amount]))
            }
            #[cfg(feature = "component-transfer")]
            Self::Opacity(amount) => Primitive::ComponentTransfer(ComponentTransfer {
                alpha: TransferFunction::Table(vec![0.0, amount.min(1.0)]),
                ..ComponentTransfer::default()
            }),
            #[cfg(feature = "color-matrix")]
            Self::Saturate(amount) => Primitive::ColorMatrix(ColorMatrix::Saturate(amount)),
            #[cfg(feature = "color-matrix")]
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

impl FilterFunction {
    /// `region` grown by what the function draws beyond it.
    pub(crate) fn extend(&self, region: Rect) -> Rect {
        match *self {
            // A blur visibly extends the content by about 3 standard deviations on each side.
            #[cfg(feature = "gaussian-blur")]
            Self::Blur(std_dev) => region.outset(3.0 * std_dev.max(0.0)),
            #[cfg(feature = "drop-shadow")]
            Self::DropShadow {
                dx, dy, std_dev, ..
            } => {
                let mut shadow = region.outset(3.0 * std_dev.max(0.0));
                shadow.x += dx;
                shadow.y += dy;
                region.union(shadow)
            }
            #[cfg(feature = "component-transfer")]
            Self::Brightness(_) | Self::Contrast(_) | Self::Invert(_) | Self::Opacity(_) => region,
            #[cfg(feature = "color-matrix")]
            Self::Grayscale(_) | Self::HueRotate(_) | Self::Saturate(_) | Self::Sepia(_) => region,
        }
    }
}

#[cfg(feature = "component-transfer")]
fn rgb_transfer<I>(function: TransferFunction) -> Primitive<I> {
    Primitive::ComponentTransfer(ComponentTransfer {
        red: function.clone(),
        green: function.clone(),
        blue: function,
        alpha: TransferFunction::Identity,
    })
}
