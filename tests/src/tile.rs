//! Filter primitive feTile (§9.20).

use crate::check;

#[test]
fn empty_input() {
    check("tile/empty_input");
}

#[test]
fn huge_input() {
    check("tile/huge_input");
}

#[test]
fn inherited_cell() {
    check("tile/inherited_cell");
}

#[test]
fn input_outside_region() {
    check("tile/input_outside_region");
}

#[test]
fn simple() {
    check("tile/simple");
}

#[test]
fn single_pixel() {
    check("tile/single_pixel");
}

#[test]
fn source_graphic() {
    check("tile/source_graphic");
}

#[test]
fn subregion() {
    check("tile/subregion");
}

#[test]
fn tile_origin() {
    check("tile/tile_origin");
}
