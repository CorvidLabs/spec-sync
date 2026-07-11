## ADDED

### REQUIREMENT REQ-exports-001
The Rust export scanner SHALL preserve every documented contract symbol across every source file listed by a spec.

Acceptance Criteria
- Regex and AST parsing include plain `pub` and crate-visible `pub(crate)` declarations, including valid whitespace variants.
- Crate-visible items and re-exports inside private inline modules are included consistently in both parse modes.
- Narrower `pub(super)`, `pub(self)`, and `pub(in ...)` declarations remain excluded.
- A multi-file fixture matching issue #334 passes strict phantom/undocumented export validation in both parse modes.
