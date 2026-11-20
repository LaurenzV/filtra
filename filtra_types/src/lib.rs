//! Types describing SVG and CSS filter effects.
//!
//! This crate provides the vocabulary to describe filter primitives and the chains and graphs they
//! form, as defined in [Filter Effects Module Level 1](https://drafts.fxtf.org/filter-effects-1/).

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
