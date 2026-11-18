//! Filter primitive feConvolveMatrix (§9.9).

use crate::check;

#[test]
fn bias() {
    check("convolve_matrix/bias");
}

#[test]
fn bias_preserve_alpha() {
    check("convolve_matrix/bias_preserve_alpha");
}

#[test]
fn blur() {
    check("convolve_matrix/blur");
}

#[test]
fn clamped() {
    check("convolve_matrix/clamped");
}

#[test]
fn divisor() {
    check("convolve_matrix/divisor");
}

#[test]
fn divisor_zero() {
    check("convolve_matrix/divisor_zero");
}

#[test]
fn edge_default() {
    check("convolve_matrix/edge_default");
}

#[test]
fn edge_detection() {
    check("convolve_matrix/edge_detection");
}

#[test]
fn edge_duplicate() {
    check("convolve_matrix/edge_duplicate");
}

#[test]
fn edge_mirror() {
    check("convolve_matrix/edge_mirror");
}

#[test]
fn edge_none() {
    check("convolve_matrix/edge_none");
}

#[test]
fn edge_wrap() {
    check("convolve_matrix/edge_wrap");
}

#[test]
fn emboss() {
    check("convolve_matrix/emboss");
}

#[test]
fn empty_subregion() {
    check("convolve_matrix/empty_subregion");
}

#[test]
fn even_order() {
    check("convolve_matrix/even_order");
}

#[test]
fn input_subregion() {
    check("convolve_matrix/input_subregion");
}

#[test]
fn input_subregion_wrap() {
    check("convolve_matrix/input_subregion_wrap");
}

#[test]
fn large() {
    check("convolve_matrix/large");
}

#[test]
fn mirror_beyond() {
    check("convolve_matrix/mirror_beyond");
}

#[test]
fn negative_bias() {
    check("convolve_matrix/negative_bias");
}

#[test]
fn negative_divisor() {
    check("convolve_matrix/negative_divisor");
}

#[test]
fn no_kernel() {
    check("convolve_matrix/no_kernel");
}

#[test]
fn order_fractional() {
    check("convolve_matrix/order_fractional");
}

#[test]
fn order_negative() {
    check("convolve_matrix/order_negative");
}

#[test]
fn order_one_zero() {
    check("convolve_matrix/order_one_zero");
}

#[test]
fn order_three_values() {
    check("convolve_matrix/order_three_values");
}

#[test]
fn order_zero() {
    check("convolve_matrix/order_zero");
}

#[test]
fn preserve_alpha() {
    check("convolve_matrix/preserve_alpha");
}

#[test]
fn preserve_alpha_wrap() {
    check("convolve_matrix/preserve_alpha_wrap");
}

#[test]
fn rectangular() {
    check("convolve_matrix/rectangular");
}

#[test]
fn region_wrap() {
    check("convolve_matrix/region_wrap");
}

#[test]
fn rotation() {
    check("convolve_matrix/rotation");
}

#[test]
fn sharpen() {
    check("convolve_matrix/sharpen");
}

#[test]
fn single() {
    check("convolve_matrix/single");
}

#[test]
fn source_alpha() {
    check("convolve_matrix/source_alpha");
}

#[test]
fn srgb() {
    check("convolve_matrix/srgb");
}

#[test]
fn subregion() {
    check("convolve_matrix/subregion");
}

#[test]
fn tall_duplicate() {
    check("convolve_matrix/tall_duplicate");
}

#[test]
fn tall_mirror() {
    check("convolve_matrix/tall_mirror");
}

#[test]
fn tall_none() {
    check("convolve_matrix/tall_none");
}

#[test]
fn tall_wrap() {
    check("convolve_matrix/tall_wrap");
}

#[test]
fn target() {
    check("convolve_matrix/target");
}

#[test]
fn target_outside() {
    check("convolve_matrix/target_outside");
}

#[test]
fn target_partial() {
    check("convolve_matrix/target_partial");
}

#[test]
fn target_y() {
    check("convolve_matrix/target_y");
}

#[test]
fn vertical_mirror() {
    check("convolve_matrix/vertical_mirror");
}

#[test]
fn vertical_none() {
    check("convolve_matrix/vertical_none");
}

#[test]
fn vertical_wrap() {
    check("convolve_matrix/vertical_wrap");
}

#[test]
fn wrap_beyond() {
    check("convolve_matrix/wrap_beyond");
}

#[test]
fn wrong_count() {
    check("convolve_matrix/wrong_count");
}

#[test]
fn wrong_count_subregion() {
    check("convolve_matrix/wrong_count_subregion");
}

#[test]
fn zero_sum() {
    check("convolve_matrix/zero_sum");
}
