---
change: CHG-0016-preserve-c-ast-behavior-under-rust-1-95-clippy
artifact: plan
---

# Plan

1. Move the existing public-visibility condition onto the `friend_declaration` match arm.
2. Leave the traversal call and all other branches unchanged.
3. Run formatting, Clippy with warnings denied, focused C++ AST tests, the full Rust suite, strict SpecSync validation, and Trust verification.
4. Review the final diff to confirm it contains only the mechanical syntax correction and lifecycle evidence.
