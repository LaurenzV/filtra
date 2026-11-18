//! Filter primitive feDropShadow (§9.12).

use crate::check;

#[test]
fn default() {
    check("drop_shadow/default");
}

#[test]
fn empty_subregion() {
    check("drop_shadow/empty_subregion");
}

#[test]
fn far() {
    check("drop_shadow/far");
}

#[test]
fn fractional_offset() {
    check("drop_shadow/fractional_offset");
}

#[test]
fn input_subregion() {
    check("drop_shadow/input_subregion");
}

#[test]
fn invalid() {
    check("drop_shadow/invalid");
}

#[test]
fn linear_rgb() {
    check("drop_shadow/linear_rgb");
}

#[test]
fn negative() {
    check("drop_shadow/negative");
}

#[test]
fn negative_offset() {
    check("drop_shadow/negative_offset");
}

#[test]
fn node_input() {
    check("drop_shadow/node_input");
}

#[test]
fn opacity() {
    check("drop_shadow/opacity");
}

#[test]
fn per_axis() {
    check("drop_shadow/per_axis");
}

#[test]
fn region() {
    check("drop_shadow/region");
}

#[test]
fn simple() {
    check("drop_shadow/simple");
}

#[test]
fn source_alpha() {
    check("drop_shadow/source_alpha");
}

#[test]
fn srgb() {
    check("drop_shadow/srgb");
}

#[test]
fn subregion() {
    check("drop_shadow/subregion");
}

#[test]
fn zero() {
    check("drop_shadow/zero");
}
