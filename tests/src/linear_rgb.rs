//! The `color-interpolation-filters` property (§10) with linearRGB.

use crate::check;

#[test]
fn auto() {
    check("linear_rgb/auto");
}

#[test]
fn color_matrix() {
    check("linear_rgb/color_matrix");
}

#[test]
fn composite_arithmetic() {
    check("linear_rgb/composite_arithmetic");
}

#[test]
fn default() {
    check("linear_rgb/default");
}

#[test]
fn merge() {
    check("linear_rgb/merge");
}

#[test]
fn offset_and_tile() {
    check("linear_rgb/offset_and_tile");
}

#[test]
fn saturate() {
    check("linear_rgb/saturate");
}
