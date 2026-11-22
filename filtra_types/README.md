# filtra_types

[![crates.io](https://img.shields.io/crates/v/filtra_types.svg)](https://crates.io/crates/filtra_types)
[![docs.rs](https://img.shields.io/docsrs/filtra_types.svg)](https://docs.rs/filtra_types)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)

Types describing SVG and CSS filter effects.

This crate provides the vocabulary to describe filter primitives and the chains and graphs they
form, as defined in [Filter Effects Module Level 1](https://drafts.fxtf.org/filter-effects-1/).

## Features

Each filter primitive has a feature of the same name in kebab case, such as `gaussian-blur`, which
enables its type. `all`, the default, enables every primitive, and `css-functions` enables the ones
that `FilterFunction` needs. `serde` implements `Serialize` and `Deserialize` for every type.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
