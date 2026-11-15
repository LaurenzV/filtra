//! Types describing SVG and CSS filter effects.

#![no_std]

extern crate alloc;

mod color;
mod primitive;

pub use color::{Color, ColorInterpolation};
pub use primitive::*;
