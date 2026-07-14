---
change: CHG-0031-keep-the-rustsec-audit-job-compatible-with-the-rust-1-89-msrv
artifact: plan
---

# Plan

1. Install the repository's Rust 1.89 toolchain in the audit job.
2. Install `cargo-audit 0.22.2` with `--locked` so its tested dependency graph is preserved.
3. Run `cargo audit` as the same blocking vulnerability gate.
4. Validate the workflow locally and require the hosted audit and aggregate CI checks to pass.
