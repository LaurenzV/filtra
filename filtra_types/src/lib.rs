//! Types describing SVG and CSS filter effects.

#![no_std]

extern crate alloc;

mod color;
mod filter;
#[cfg(any(
    feature = "gaussian-blur",
    feature = "drop-shadow",
    feature = "component-transfer",
    feature = "color-matrix"
))]
mod filter_function;
mod primitive;

pub use color::{Color, ColorInterpolation};
pub use filter::{Filter, Input, Node, Rect, Subregion};
#[cfg(any(
    feature = "gaussian-blur",
    feature = "drop-shadow",
    feature = "component-transfer",
    feature = "color-matrix"
))]
pub use filter_function::FilterFunction;
pub use primitive::*;
