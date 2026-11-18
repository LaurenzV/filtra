//! Filter primitive feSpecularLighting (§9.19).

use crate::check;

#[test]
fn color() {
    check("specular_lighting/color");
}

#[test]
fn composite() {
    check("specular_lighting/composite");
}

#[test]
fn constant() {
    check("specular_lighting/constant");
}

#[test]
fn default() {
    check("specular_lighting/default");
}

#[test]
fn distant() {
    check("specular_lighting/distant");
}

#[test]
fn exponent_half() {
    check("specular_lighting/exponent_half");
}

#[test]
fn exponent_large() {
    check("specular_lighting/exponent_large");
}

#[test]
fn exponent_negative() {
    check("specular_lighting/exponent_negative");
}

#[test]
fn exponent_zero() {
    check("specular_lighting/exponent_zero");
}

#[test]
fn negative_constant() {
    check("specular_lighting/negative_constant");
}

#[test]
fn point() {
    check("specular_lighting/point");
}

#[test]
fn point_default() {
    check("specular_lighting/point_default");
}

#[test]
fn premultiplied() {
    check("specular_lighting/premultiplied");
}

#[test]
fn spot() {
    check("specular_lighting/spot");
}

#[test]
fn spot_no_cone() {
    check("specular_lighting/spot_no_cone");
}

#[test]
fn srgb() {
    check("specular_lighting/srgb");
}

#[test]
fn subregion() {
    check("specular_lighting/subregion");
}
