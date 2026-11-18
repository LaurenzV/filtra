//! Filter primitive feDiffuseLighting (§9.10).

use crate::check;

#[test]
fn color() {
    check("diffuse_lighting/color");
}

#[test]
fn constant() {
    check("diffuse_lighting/constant");
}

#[test]
fn default() {
    check("diffuse_lighting/default");
}

#[test]
fn distant() {
    check("diffuse_lighting/distant");
}

#[test]
fn input_subregion() {
    check("diffuse_lighting/input_subregion");
}

#[test]
fn negative_constant() {
    check("diffuse_lighting/negative_constant");
}

#[test]
fn negative_surface_scale() {
    check("diffuse_lighting/negative_surface_scale");
}

#[test]
fn point() {
    check("diffuse_lighting/point");
}

#[test]
fn point_on_surface() {
    check("diffuse_lighting/point_on_surface");
}

#[test]
fn sharp() {
    check("diffuse_lighting/sharp");
}

#[test]
fn source_alpha() {
    check("diffuse_lighting/source_alpha");
}

#[test]
fn spot() {
    check("diffuse_lighting/spot");
}

#[test]
fn spot_at_itself() {
    check("diffuse_lighting/spot_at_itself");
}

#[test]
fn spot_exponent() {
    check("diffuse_lighting/spot_exponent");
}

#[test]
fn spot_exponent_negative() {
    check("diffuse_lighting/spot_exponent_negative");
}

#[test]
fn spot_exponent_zero() {
    check("diffuse_lighting/spot_exponent_zero");
}

#[test]
fn spot_narrow() {
    check("diffuse_lighting/spot_narrow");
}

#[test]
fn spot_negative_cone() {
    check("diffuse_lighting/spot_negative_cone");
}

#[test]
fn spot_no_cone() {
    check("diffuse_lighting/spot_no_cone");
}

#[test]
fn spot_wide_cone() {
    check("diffuse_lighting/spot_wide_cone");
}

#[test]
fn srgb() {
    check("diffuse_lighting/srgb");
}

#[test]
fn subregion() {
    check("diffuse_lighting/subregion");
}

#[test]
fn subregion_column() {
    check("diffuse_lighting/subregion_column");
}

#[test]
fn subregion_pixel() {
    check("diffuse_lighting/subregion_pixel");
}

#[test]
fn subregion_row() {
    check("diffuse_lighting/subregion_row");
}

#[test]
fn zero_surface_scale() {
    check("diffuse_lighting/zero_surface_scale");
}
