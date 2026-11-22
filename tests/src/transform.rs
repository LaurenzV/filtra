//! The mapping from user space to pixels, with the content scaled to twice its size.

use crate::check;

#[test]
fn blur() {
    check("transform/blur");
}

#[test]
fn convolve_matrix() {
    check("transform/convolve_matrix");
}

#[test]
fn displacement_map() {
    check("transform/displacement_map");
}

#[test]
fn drop_shadow() {
    check("transform/drop_shadow");
}

#[test]
fn image() {
    check("transform/image");
}

#[test]
fn lighting() {
    check("transform/lighting");
}

#[test]
fn lighting_spot() {
    check("transform/lighting_spot");
}

#[test]
fn morphology() {
    check("transform/morphology");
}

#[test]
fn offset() {
    check("transform/offset");
}

#[test]
fn subregion() {
    check("transform/subregion");
}

#[test]
fn tile() {
    check("transform/tile");
}

#[test]
fn turbulence() {
    check("transform/turbulence");
}
