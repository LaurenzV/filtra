//! The filter region (§8).

use crate::check;

#[test]
fn huge() {
    check("region/huge");
}

#[test]
fn negative_height() {
    check("region/negative_height");
}

#[test]
fn zero_width() {
    check("region/zero_width");
}
