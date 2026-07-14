---
id: CHG-0031-keep-the-rustsec-audit-job-compatible-with-the-rust-1-89-msrv
state: archived
type: operations
base_commit: e5c6829df0d65a1b3ae18b2d1dccd47c422b9208
---

# Keep the RustSec audit job compatible with the Rust 1.89 MSRV

## Intent

Keep the RustSec audit job compatible with the Rust 1.89 MSRV

## Affected Canonical Specs

- None

## Acceptance Criteria

- The audit job installs cargo-audit 0.22.2 with its lockfile under Rust 1.89
- executes cargo audit without weakening advisory checks
- and the required CI gate passes.

## No-spec Rationale

This changes only CI tool installation wiring; the security audit policy and SpecSync product contract remain unchanged.
