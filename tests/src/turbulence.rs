//! Filter primitive feTurbulence (§9.21).

use crate::check;

#[test]
fn anisotropic() {
    check("turbulence/anisotropic");
}

#[test]
fn fractal_noise() {
    check("turbulence/fractal_noise");
}

#[test]
fn huge_frequency() {
    check("turbulence/huge_frequency");
}

#[test]
fn huge_frequency_stitch() {
    check("turbulence/huge_frequency_stitch");
}

#[test]
fn many_octaves() {
    check("turbulence/many_octaves");
}

#[test]
fn negative_frequency() {
    check("turbulence/negative_frequency");
}

#[test]
fn negative_frequency_one_axis() {
    check("turbulence/negative_frequency_one_axis");
}

#[test]
fn negative_octaves() {
    check("turbulence/negative_octaves");
}

#[test]
fn no_stitch() {
    check("turbulence/no_stitch");
}

#[test]
fn octaves() {
    check("turbulence/octaves");
}

#[test]
fn seed() {
    check("turbulence/seed");
}

#[test]
fn seed_fractional() {
    check("turbulence/seed_fractional");
}

#[test]
fn seed_negative() {
    check("turbulence/seed_negative");
}

#[test]
fn srgb() {
    check("turbulence/srgb");
}

#[test]
fn stitch() {
    check("turbulence/stitch");
}

#[test]
fn subregion() {
    check("turbulence/subregion");
}

#[test]
fn turbulence() {
    check("turbulence/turbulence");
}

#[test]
fn zero_frequency() {
    check("turbulence/zero_frequency");
}

#[test]
fn zero_octaves() {
    check("turbulence/zero_octaves");
}
