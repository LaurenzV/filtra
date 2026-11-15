//! Filter primitive feColorMatrix (§9.6).

use crate::check;

#[test]
fn hue_rotate() {
    check("color_matrix/hue_rotate");
}

#[test]
fn luminance_to_alpha() {
    check("color_matrix/luminance_to_alpha");
}

#[test]
fn matrix() {
    check("color_matrix/matrix");
}

#[test]
fn matrix_alpha_column() {
    check("color_matrix/matrix_alpha_column");
}

#[test]
fn matrix_alpha_row() {
    check("color_matrix/matrix_alpha_row");
}

#[test]
fn matrix_clamped() {
    check("color_matrix/matrix_clamped");
}

#[test]
fn matrix_default() {
    check("color_matrix/matrix_default");
}

#[test]
fn matrix_wrong_count() {
    check("color_matrix/matrix_wrong_count");
}

#[test]
fn saturate() {
    check("color_matrix/saturate");
}

#[test]
fn saturate_default() {
    check("color_matrix/saturate_default");
}
