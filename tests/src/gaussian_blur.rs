//! Filter primitive feGaussianBlur (§9.14).

use crate::{check, check_at};

#[test]
fn default() {
    check("gaussian_blur/default");
}

#[test]
fn different_values() {
    check("gaussian_blur/different_values");
}

#[test]
fn empty_subregion() {
    check("gaussian_blur/empty_subregion");
}

#[test]
fn far() {
    check("gaussian_blur/far");
}

#[test]
fn huge() {
    check("gaussian_blur/huge");
}

#[test]
fn infinite() {
    check_at("gaussian_blur/infinite", 2);
}

#[test]
fn input_subregion() {
    check("gaussian_blur/input_subregion");
}

#[test]
fn invalid() {
    check("gaussian_blur/invalid");
}

#[test]
fn large() {
    check("gaussian_blur/large");
}

#[test]
fn negative() {
    check("gaussian_blur/negative");
}

#[test]
fn negative_x() {
    check("gaussian_blur/negative_x");
}

#[test]
fn odd_size() {
    check("gaussian_blur/odd_size");
}

#[test]
fn region() {
    check("gaussian_blur/region");
}

#[test]
fn shadow() {
    check("gaussian_blur/shadow");
}

#[test]
fn simple() {
    check("gaussian_blur/simple");
}

#[test]
fn sizes() {
    check("gaussian_blur/sizes");
}

#[test]
fn small() {
    check("gaussian_blur/small");
}

#[test]
fn source_alpha() {
    check("gaussian_blur/source_alpha");
}

#[test]
fn srgb() {
    check("gaussian_blur/srgb");
}

#[test]
fn subregion() {
    check("gaussian_blur/subregion");
}

#[test]
fn tiny() {
    check("gaussian_blur/tiny");
}

#[test]
fn zero() {
    check("gaussian_blur/zero");
}

#[test]
fn zero_x() {
    check("gaussian_blur/zero_x");
}

#[test]
fn zero_y() {
    check("gaussian_blur/zero_y");
}
