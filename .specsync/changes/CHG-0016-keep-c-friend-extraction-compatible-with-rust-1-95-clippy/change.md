---
id: CHG-0016-keep-c-friend-extraction-compatible-with-rust-1-95-clippy
state: accepted
type: refactor
base_commit: 60bd655c2365addc3d7a37e95f5fc20c06a746ff
---

# Keep C++ friend extraction compatible with Rust 1.95 Clippy

## Intent

Keep C++ friend extraction compatible with Rust 1.95 Clippy

## Affected Canonical Specs

- None

## Acceptance Criteria

- Rust 1.95 Clippy passes with warnings denied; all C++ extractor tests pass unchanged

## No-spec Rationale

The guarded match is equivalent to the prior nested visibility condition and changes no extraction behavior or public contract.
