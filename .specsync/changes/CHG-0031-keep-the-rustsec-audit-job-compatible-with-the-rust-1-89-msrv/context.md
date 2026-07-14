---
change: CHG-0031-keep-the-rustsec-audit-job-compatible-with-the-rust-1-89-msrv
artifact: context
---

# Context

PR #370's `audit` job failed before scanning the repository. `rustsec/audit-check@v2.0.0`
resolved `cargo-audit 0.22.2` without its published lockfile, selecting `kstring 2.0.3`, which
requires Rust 1.96. The repository intentionally validates its Rust 1.89 minimum supported version.
The Cargo error itself recommends reinstalling with `--locked`.

## Implementation Status

The audit job now installs the pinned Rust 1.89 toolchain, installs `cargo-audit 0.22.2`
with `--locked`, and runs `cargo audit` directly. `actionlint` passes for the updated workflow.
The same `cargo-audit 0.22.2` executable also completed under Rust 1.89 locally, scanning all
218 locked dependencies without a vulnerability failure. It retained the existing allowed
unmaintained-crate warning for `instant`. The hosted audit and aggregate required gate remain to
be confirmed after the change is pushed.
