---
change: CHG-0031-keep-the-rustsec-audit-job-compatible-with-the-rust-1-89-msrv
artifact: design
---

# Design

The audit job remains an independent required CI job. Its sequence is checkout, install the Rust
1.89 toolchain, install the pinned audit executable from its locked dependency graph, and scan the
committed `Cargo.lock`. No vulnerability ignore, severity threshold, or aggregate-gate behavior is
changed.
