//! Filter primitive feComponentTransfer (§9.7).

use crate::check;

#[test]
fn discrete() {
    check("component_transfer/discrete");
}

#[test]
fn gamma() {
    check("component_transfer/gamma");
}

#[test]
fn gamma_alpha() {
    check("component_transfer/gamma_alpha");
}

#[test]
fn identity() {
    check("component_transfer/identity");
}

#[test]
fn last_wins() {
    check("component_transfer/last_wins");
}

#[test]
fn linear() {
    check("component_transfer/linear");
}

#[test]
fn linear_alpha() {
    check("component_transfer/linear_alpha");
}

#[test]
fn table() {
    check("component_transfer/table");
}

#[test]
fn table_alpha() {
    check("component_transfer/table_alpha");
}

#[test]
fn table_empty() {
    check("component_transfer/table_empty");
}

#[test]
fn table_single() {
    check("component_transfer/table_single");
}
