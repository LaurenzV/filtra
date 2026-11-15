//! Filter primitive feOffset (§9.18).

use crate::check;

#[test]
fn beyond_region() {
    check("offset/beyond_region");
}

#[test]
fn clipped_input() {
    check("offset/clipped_input");
}

#[test]
fn dx_only() {
    check("offset/dx_only");
}

#[test]
fn dy_only() {
    check("offset/dy_only");
}

#[test]
fn fractional() {
    check("offset/fractional");
}

#[test]
fn huge_negative() {
    check("offset/huge_negative");
}

#[test]
fn huge_positive() {
    check("offset/huge_positive");
}

#[test]
fn inherits_input_subregion() {
    check("offset/inherits_input_subregion");
}

#[test]
fn negative() {
    check("offset/negative");
}

#[test]
fn none() {
    check("offset/none");
}

#[test]
fn positive() {
    check("offset/positive");
}

#[test]
fn source_alpha() {
    check("offset/source_alpha");
}

#[test]
fn subregion() {
    check("offset/subregion");
}
