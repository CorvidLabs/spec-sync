---
id: CHG-0016-preserve-c-ast-behavior-under-rust-1-95-clippy
state: archived
type: refactor
base_commit: 60bd655c2365addc3d7a37e95f5fc20c06a746ff
---

# Preserve C++ AST behavior under Rust 1.95 Clippy

## Intent

Preserve C++ AST behavior under Rust 1.95 Clippy

## Affected Canonical Specs

- None

## Acceptance Criteria

- Rust 1.95 Clippy passes with warnings denied; C++ AST unit tests remain green; the diff changes only match syntax and lifecycle evidence; no exported symbols or runtime behavior change

## No-spec Rationale

This mechanically collapses a match guard required by Rust 1.95 Clippy without changing C++ AST traversal, exports, or any public contract.
