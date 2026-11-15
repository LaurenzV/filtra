//! Types describing SVG and CSS filter effects.

#![no_std]

extern crate alloc;

mod color;
mod filter;
mod filter_function;
mod primitive;

pub use color::{Color, ColorInterpolation};
pub use filter::{Filter, Input, Node, Rect, Subregion};
pub use filter_function::FilterFunction;
pub use primitive::*;
