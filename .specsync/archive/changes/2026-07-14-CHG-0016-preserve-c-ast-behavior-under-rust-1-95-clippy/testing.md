---
change: CHG-0016-preserve-c-ast-behavior-under-rust-1-95-clippy
artifact: testing
---

# Testing

## Behavioral Evidence

- `fledge run lint` passes on Rust 1.95 with `-D warnings`.
- Focused C++ AST tests preserve public friend extraction and non-public member exclusion.
- The complete Rust test suite remains green.
- Strict SpecSync and Trust gates pass with the approved no-spec-change rationale.

## Diff Audit

The implementation diff must only move `vis == Visibility::Public` from the nested `if` to the existing match guard and remove the now-redundant block. There must be no changed traversal call, symbol extraction, fixtures, canonical specs, or public documentation.
