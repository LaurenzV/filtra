//! Filter primitive feComposite (§9.8).

use crate::check;

#[test]
fn arithmetic() {
    check("composite/arithmetic");
}

#[test]
fn arithmetic_clamped() {
    check("composite/arithmetic_clamped");
}

#[test]
fn arithmetic_default() {
    check("composite/arithmetic_default");
}

#[test]
fn atop() {
    check("composite/atop");
}

#[test]
fn default() {
    check("composite/default");
}

#[test]
fn in_() {
    check("composite/in");
}

#[test]
fn lighter() {
    check("composite/lighter");
}

#[test]
fn out() {
    check("composite/out");
}

#[test]
fn over() {
    check("composite/over");
}

#[test]
fn subregion() {
    check("composite/subregion");
}

#[test]
fn xor() {
    check("composite/xor");
}
