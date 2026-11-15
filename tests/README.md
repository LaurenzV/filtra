# filtra tests

Each test is an SVG file in `svg/`. Its shapes are rendered with Vello CPU, its filter is applied
with filtra, and the result, composited onto white, must have the same pixels as the image at the
same path in `references/`. Tests in `transform/` are rendered at twice the size of the SVG; all
others at their size.

```bash
cargo test -p filtra_tests
```

On a mismatch, filtra's result is written to `target/filtra-diffs/`. To replace the references
with filtra's current results, after checking them against the specification and browsers:

```bash
FILTRA_REPLACE=1 cargo test -p filtra_tests
oxipng -o max --strip safe -r tests/references
```

The SVG files use a minimal subset of SVG (shapes with solid fills in one `<g>`, filtered by one
`<filter>` in user space or by filter functions), so they render the same in browsers and other
SVG renderers.

The filter only sees the canvas. Content outside of it, like in resvg's
`filter/content-outside-the-canvas` tests, is the renderer's concern: it has to pass a larger
source.
