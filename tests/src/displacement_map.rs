//! Filter primitive feDisplacementMap (§9.11).

use crate::check;

#[test]
fn channels() {
    check("displacement_map/channels");
}

#[test]
fn default_channels() {
    check("displacement_map/default_channels");
}

#[test]
fn half_pixel() {
    check("displacement_map/half_pixel");
}

#[test]
fn huge_scale() {
    check("displacement_map/huge_scale");
}

#[test]
fn linear_rgb_map() {
    check("displacement_map/linear_rgb_map");
}

#[test]
fn negative_scale() {
    check("displacement_map/negative_scale");
}

#[test]
fn outside_input() {
    check("displacement_map/outside_input");
}

#[test]
fn outside_map() {
    check("displacement_map/outside_map");
}

#[test]
fn srgb_map() {
    check("displacement_map/srgb_map");
}

#[test]
fn translucent_map() {
    check("displacement_map/translucent_map");
}

#[test]
fn turbulence() {
    check("displacement_map/turbulence");
}

#[test]
fn zero_scale() {
    check("displacement_map/zero_scale");
}
