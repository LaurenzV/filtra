//! Filter primitive feMorphology (§9.17).

use crate::check;

#[test]
fn below_half() {
    check("morphology/below_half");
}

#[test]
fn dilate() {
    check("morphology/dilate");
}

#[test]
fn erode() {
    check("morphology/erode");
}

#[test]
fn fractional() {
    check("morphology/fractional");
}

#[test]
fn huge_dilate() {
    check("morphology/huge_dilate");
}

#[test]
fn huge_erode() {
    check("morphology/huge_erode");
}

#[test]
fn negative_radius() {
    check("morphology/negative_radius");
}

#[test]
fn no_radius() {
    check("morphology/no_radius");
}

#[test]
fn one_radius_zero() {
    check("morphology/one_radius_zero");
}

#[test]
fn radius_xy() {
    check("morphology/radius_xy");
}

#[test]
fn region_edge() {
    check("morphology/region_edge");
}

#[test]
fn source_alpha() {
    check("morphology/source_alpha");
}

#[test]
fn subregion() {
    check("morphology/subregion");
}

#[test]
fn too_many_values() {
    check("morphology/too_many_values");
}

#[test]
fn zero_radius() {
    check("morphology/zero_radius");
}
