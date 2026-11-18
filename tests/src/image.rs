//! Filter primitive feImage (§9.15), referencing elements that the test renders.

use crate::check;

#[test]
fn blur() {
    check("image/blur");
}

#[test]
fn empty_subregion() {
    check("image/empty_subregion");
}

#[test]
fn linear_rgb() {
    check("image/linear_rgb");
}

#[test]
fn merge() {
    check("image/merge");
}

#[test]
fn region() {
    check("image/region");
}

#[test]
fn reused_buffer() {
    check("image/reused_buffer");
}

#[test]
fn several() {
    check("image/several");
}

#[test]
fn simple() {
    check("image/simple");
}

#[test]
fn subregion() {
    check("image/subregion");
}

#[test]
fn tile() {
    check("image/tile");
}

#[test]
fn unused() {
    check("image/unused");
}
