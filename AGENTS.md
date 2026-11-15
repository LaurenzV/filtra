# AGENTS.md

## Implementation
- All code must be grounded in the respective specifications (Filter Effects Module Level 1 (or SVG 1.1 if not specified), Compositing and Blending Level 1, CSS Color 4).
- Comparing rendering against other implementations is fine, but the spec is the final source of truth, browsers can also get things wrong.
- Do not copy code from other libraries, everything must be implemented from scratch and spec-driven.
- It is not required to follow the spec's algorithm line-by-line, if other
  algorithms are possible that achieve the same effect up to rounding but are faster/more memory
  efficient.

## Code
- Avoid comments by default. If you do add comments, keep them brief, and only to provide background information (for example explaining what algorithm we use for certain code). Don't narrate code or fixes, inline comments only for code where the reasoning behind it is not possible to derive just by looking at it.
- Doc comments only on public API: one short line, describing that item only (not what other code
  does with it).
- Order items in a file by importance: public items first, then `pub(crate)`, then private ones,
  and by importance within each group.

## Commits
- For commits that apply to a specific crate, prefix the commit message with `crate: {message}`.
- Use backticks for crate names, variable names etc.
