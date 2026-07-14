---
id: CHG-0030-satisfy-rust-1-95-clippy-for-the-cargo-lifecycle-verification-guard
state: archived
type: bug_fix
base_commit: c98d29810f78abcdd6a2fec9b137667d3ab2fc5b
---

# Satisfy Rust 1.95 Clippy for the Cargo lifecycle verification guard

## Intent

Satisfy Rust 1.95 Clippy for the Cargo lifecycle verification guard

## Affected Canonical Specs

- None

## Acceptance Criteria

- Rust 1.95 Clippy passes and Cargo lifecycle verification behavior remains unchanged.

## No-spec Rationale

This is a Clippy-required boolean-expression simplification with identical lifecycle behavior and no canonical contract change.
