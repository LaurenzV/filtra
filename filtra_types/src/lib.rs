//! Types describing SVG and CSS filter effects.
//!
//! This crate provides the vocabulary to describe filter primitives and the chains and graphs they
//! form, as defined in [Filter Effects Module Level 1](https://drafts.fxtf.org/filter-effects-1/).
//!
//! # Features
//!
//! Each filter primitive has a feature of the same name in kebab case, such as `gaussian-blur`,
//! which enables its type. `all`, the default, enables every primitive, and `css-functions` enables
//! the ones that `FilterFunction` needs. `serde` implements `Serialize` and `Deserialize` for every
//! type.

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]

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
