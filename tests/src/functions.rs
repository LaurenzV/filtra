//! Filter functions (§6).

use crate::check;

#[test]
fn brightness() {
    check("functions/brightness");
}

#[test]
fn brightness_half() {
    check("functions/brightness_half");
}

#[test]
fn brightness_zero() {
    check("functions/brightness_zero");
}

#[test]
fn chain() {
    check("functions/chain");
}

#[test]
fn chain_reversed() {
    check("functions/chain_reversed");
}

#[test]
fn contrast() {
    check("functions/contrast");
}

#[test]
fn contrast_half() {
    check("functions/contrast_half");
}

#[test]
fn contrast_zero() {
    check("functions/contrast_zero");
}

#[test]
fn grayscale() {
    check("functions/grayscale");
}

#[test]
fn grayscale_above_one() {
    check("functions/grayscale_above_one");
}

#[test]
fn grayscale_default() {
    check("functions/grayscale_default");
}

#[test]
fn grayscale_half() {
    check("functions/grayscale_half");
}

#[test]
fn grayscale_number() {
    check("functions/grayscale_number");
}

#[test]
fn grayscale_zero() {
    check("functions/grayscale_zero");
}

#[test]
fn hue_rotate() {
    check("functions/hue_rotate");
}

#[test]
fn hue_rotate_beyond_360() {
    check("functions/hue_rotate_beyond_360");
}

#[test]
fn hue_rotate_default() {
    check("functions/hue_rotate_default");
}

#[test]
fn hue_rotate_negative() {
    check("functions/hue_rotate_negative");
}

#[test]
fn hue_rotate_turn() {
    check("functions/hue_rotate_turn");
}

#[test]
fn identity() {
    check("functions/identity");
}

#[test]
fn invert() {
    check("functions/invert");
}

#[test]
fn invert_above_one() {
    check("functions/invert_above_one");
}

#[test]
fn invert_half() {
    check("functions/invert_half");
}

#[test]
fn opacity() {
    check("functions/opacity");
}

#[test]
fn opacity_above_one() {
    check("functions/opacity_above_one");
}

#[test]
fn opacity_zero() {
    check("functions/opacity_zero");
}

#[test]
fn saturate_above_one() {
    check("functions/saturate_above_one");
}

#[test]
fn saturate_default() {
    check("functions/saturate_default");
}

#[test]
fn saturate_half() {
    check("functions/saturate_half");
}

#[test]
fn saturate_zero() {
    check("functions/saturate_zero");
}

#[test]
fn sepia() {
    check("functions/sepia");
}

#[test]
fn sepia_above_one() {
    check("functions/sepia_above_one");
}

#[test]
fn sepia_half() {
    check("functions/sepia_half");
}
