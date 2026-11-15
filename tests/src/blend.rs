//! Filter primitive feBlend (§9.5).

use crate::check;

#[test]
fn backdrop_subregion() {
    check("blend/backdrop_subregion");
}

#[test]
fn color() {
    check("blend/color");
}

#[test]
fn color_burn() {
    check("blend/color_burn");
}

#[test]
fn color_burn_black_white() {
    check("blend/color_burn_black_white");
}

#[test]
fn color_dodge() {
    check("blend/color_dodge");
}

#[test]
fn color_dodge_black_white() {
    check("blend/color_dodge_black_white");
}

#[test]
fn darken() {
    check("blend/darken");
}

#[test]
fn default() {
    check("blend/default");
}

#[test]
fn difference() {
    check("blend/difference");
}

#[test]
fn exclusion() {
    check("blend/exclusion");
}

#[test]
fn hard_light() {
    check("blend/hard_light");
}

#[test]
fn hue() {
    check("blend/hue");
}

#[test]
fn lighten() {
    check("blend/lighten");
}

#[test]
fn luminosity() {
    check("blend/luminosity");
}

#[test]
fn multiply() {
    check("blend/multiply");
}

#[test]
fn no_composite_hue() {
    check("blend/no_composite_hue");
}

#[test]
fn no_composite_multiply() {
    check("blend/no_composite_multiply");
}

#[test]
fn normal() {
    check("blend/normal");
}

#[test]
fn overlay() {
    check("blend/overlay");
}

#[test]
fn saturation() {
    check("blend/saturation");
}

#[test]
fn screen() {
    check("blend/screen");
}

#[test]
fn soft_light() {
    check("blend/soft_light");
}
