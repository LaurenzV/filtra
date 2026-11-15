//! The `color-interpolation-filters` property (§10) with linearRGB.

use crate::check;

#[test]
fn merge() {
    check("linear_rgb/merge");
}

#[test]
fn offset_and_tile() {
    check("linear_rgb/offset_and_tile");
}
