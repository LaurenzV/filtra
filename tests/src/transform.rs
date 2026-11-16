//! The mapping from user space to pixels, at twice the size of the SVG.

use crate::check_at;

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
