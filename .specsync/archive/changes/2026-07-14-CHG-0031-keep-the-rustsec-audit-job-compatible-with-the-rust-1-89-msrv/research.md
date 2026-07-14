---
change: CHG-0031-keep-the-rustsec-audit-job-compatible-with-the-rust-1-89-msrv
artifact: research
---

# Research

The failing Actions log shows `cargo-audit v0.22.2` installation selecting `kstring v2.0.3`,
followed by `rustc 1.89.0 is not supported` and Cargo's explicit recommendation to install with
`--locked`. Other repository tool-install jobs already use version pins plus `--locked`, including
the coverage job's `cargo-tarpaulin` installation.
