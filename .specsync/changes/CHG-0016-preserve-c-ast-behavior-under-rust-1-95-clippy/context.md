---
change: CHG-0016-preserve-c-ast-behavior-under-rust-1-95-clippy
artifact: context
---

# Context

Rust 1.95 adds a `collapsible_match` Clippy diagnostic for the existing `friend_declaration` arm in the C++ AST walker. With warnings denied, current `main` cannot pass its native lint lane.

The suggested match guard is semantically identical to the nested `if`: public friend declarations recurse into their child declaration, while non-public friend declarations fall through without traversal. No parser contract, exported symbol, or supported language behavior changes.
