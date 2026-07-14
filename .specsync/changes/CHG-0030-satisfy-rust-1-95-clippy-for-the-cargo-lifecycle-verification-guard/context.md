---
change: CHG-0030-satisfy-rust-1-95-clippy-for-the-cargo-lifecycle-verification-guard
artifact: context
---

# Context

The mandatory Fledge Trust lint task runs Rust 1.95 Clippy with warnings denied. It rejects the
negated `Option::is_some_and` guard added for Cargo lifecycle command classification and recommends
the equivalent `Option::is_none_or` form. No public or lifecycle behavior changes.

The implementation now uses `is_none_or`; formatting, the focused regression, and Clippy pass.
