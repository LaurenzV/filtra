//! The mapping from user space to pixels, at twice the size of the SVG.

use crate::check_at;

#[test]
fn blur() {
    check_at("transform/blur", 2);
}

#[test]
fn convolve_matrix() {
    check_at("transform/convolve_matrix", 2);
}

#[test]
fn displacement_map() {
    check_at("transform/displacement_map", 2);
}

#[test]
fn drop_shadow() {
    check_at("transform/drop_shadow", 2);
}

#[test]
fn morphology() {
    check_at("transform/morphology", 2);
}

#[test]
fn offset() {
    check_at("transform/offset", 2);
}

#[test]
fn subregion() {
    check_at("transform/subregion", 2);
}

#[test]
fn tile() {
    check_at("transform/tile", 2);
}

#[test]
fn turbulence() {
    check_at("transform/turbulence", 2);
}
