//! Filter primitive feMerge (§9.16).

use crate::check;

#[test]
fn empty() {
    check("merge/empty");
}

#[test]
fn single() {
    check("merge/single");
}

#[test]
fn subregion() {
    check("merge/subregion");
}

#[test]
fn three() {
    check("merge/three");
}

#[test]
fn two() {
    check("merge/two");
}
