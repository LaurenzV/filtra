//! Filter primitive feFlood (§9.13).

use crate::check;

#[test]
fn color() {
    check("flood/color");
}

#[test]
fn color_with_alpha() {
    check("flood/color_with_alpha");
}

#[test]
fn default() {
    check("flood/default");
}

#[test]
fn default_subregion_is_region() {
    check("flood/default_subregion_is_region");
}

#[test]
fn linear_rgb() {
    check("flood/linear_rgb");
}

#[test]
fn opacity() {
    check("flood/opacity");
}

#[test]
fn opacity_above_one() {
    check("flood/opacity_above_one");
}

#[test]
fn opacity_negative() {
    check("flood/opacity_negative");
}

#[test]
fn opacity_times_color_alpha() {
    check("flood/opacity_times_color_alpha");
}

#[test]
fn region() {
    check("flood/region");
}

#[test]
fn srgb() {
    check("flood/srgb");
}

#[test]
fn subregion() {
    check("flood/subregion");
}

#[test]
fn subregion_far_away() {
    check("flood/subregion_far_away");
}

#[test]
fn subregion_fractional() {
    check("flood/subregion_fractional");
}

#[test]
fn subregion_huge() {
    check("flood/subregion_huge");
}

#[test]
fn subregion_negative_height() {
    check("flood/subregion_negative_height");
}

#[test]
fn subregion_outside_region() {
    check("flood/subregion_outside_region");
}

#[test]
fn subregion_partial() {
    check("flood/subregion_partial");
}

#[test]
fn subregion_zero_width() {
    check("flood/subregion_zero_width");
}
