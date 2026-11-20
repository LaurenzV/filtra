# filtra

[![crates.io](https://img.shields.io/crates/v/filtra.svg)](https://crates.io/crates/filtra)
[![docs.rs](https://img.shields.io/docsrs/filtra.svg)](https://docs.rs/filtra)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)

Applies SVG and CSS filter effects to pixmaps.

This crate applies arbitrary filter graphs, as defined in
[Filter Effects Module Level 1](https://drafts.fxtf.org/filter-effects-1/), with any backend. It
provides a CPU backend, with SIMD kernels for NEON, AVX2 and WebAssembly.

## Example

The following example applies a small chain of filters to a 200×200 pixmap at 2× scale.

```rust
use filtra::types::{Filter, GaussianBlur, Input, Merge, Node, Offset, Primitive, Rect};
use filtra::Transform;
use filtra::cpu::{self, Pixmap};

let mut filter = Filter::<()>::new(Rect::new(0.0, 0.0, 100.0, 100.0));
let blur = GaussianBlur { std_dev_x: 4.0, std_dev_y: 4.0, ..Default::default() };
let blurred = filter.push(Node::new(Primitive::GaussianBlur(blur), [Input::SourceAlpha]));
let offset = Offset { dx: 5.0, dy: 5.0 };
let shadow = filter.push(Node::new(Primitive::Offset(offset), [blurred]));
filter.push(Node::new(Primitive::Merge(Merge), [shadow, Input::SourceGraphic]));

// The content, rendered at twice its size.
let mut pixmap = Pixmap::new(200, 200);
let transform = Transform::new(2.0, 2.0, 0.0, 0.0);
let mut backend = cpu::Backend::new(4);

filtra::apply(&mut backend, &filter, pixmap.as_mut(), transform, |_, _, _, _| {});
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
