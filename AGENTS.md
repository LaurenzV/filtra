# AGENTS.md

## Implementation
- All code must be grounded in the respective specifications (Filter Effects Module Level 1 (or SVG 1.1 if not specified), Compositing and Blending Level 1, CSS Color 4).
- Comparing rendering against other implementations is fine, but the spec is the final source of truth, browsers can also get things wrong.
- Do not copy code from other libraries, everything must be implemented from scratch and spec-driven.
- It is not required to follow the spec's algorithm line-by-line, if other
  algorithms are possible that achieve the same effect up to rounding but are faster/more memory
  efficient.
- In general, we always prefer approaches with better performance of color-accuracy, though we do need to make sure the deviations are within limits.

## Code
- Avoid comments. Prefer a clearer name over a comment.
- `///` on public API, and on private items only when the name and signature don't already say it.
  One short line on what the item is; no conditions, edge cases or side notes. The lib.rs
  documentation can be longer.
- `//` inside functions only for reasoning that can't be derived from the code itself, such as
  which algorithm is used and why.
- Never restate what the code does, and never describe how it changed.
- Order items in a file by importance: public items first, then `pub(crate)`, then private ones,
  and by importance within each group.
- Testing should go through visreg by default, unit tests only being used in cases where it's not possible.
- Optimize new or replaced reference images with `oxipng -o max --strip safe`.
- Follow the conventions of surrounding code in terms of when to add whitespaces between lines and when not.

## Commits
- For commits that apply to a specific crate, prefix the commit message with `crate: {message}`.
- Use backticks for crate names, variable names etc.
