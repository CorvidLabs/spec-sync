---
change: CHG-0027-make-the-hosted-rustsec-audit-bootstrap-reproducible-on-specsync-rustc-1-89-by-i
artifact: context
---

# Context

The `audit` job used `rustsec/audit-check@v2.0.0`. On July 14, 2026 that action resolved and attempted to install `cargo-audit 0.22.2` without `--locked`. Its newly selected `kstring 2.0.3` dependency requires rustc 1.96, so installation failed before any dependency audit ran on SpecSync's supported rustc 1.89 toolchain.

The repository's native audit lane already runs `cargo audit --ignore RUSTSEC-2024-0384`. Hosted CI must execute the same policy without allowing unrelated registry releases to change the installation graph.
