//! Types describing SVG and CSS filter effects.
//!
//! This crate provides the vocabulary to describe filter primitives and the chains and graphs they
//! form, as defined in [Filter Effects Module Level 1](https://drafts.fxtf.org/filter-effects-1/).
//!
//! # Features
//!
//! `serde` implements `Serialize` and `Deserialize` for every type.

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]

extern crate alloc;

mod color;
mod filter;
mod filter_function;
mod primitive;

pub use color::{Color, ColorInterpolation};
pub use filter::{Filter, Input, Node, Rect, Subregion};
pub use filter_function::FilterFunction;
pub use primitive::*;
