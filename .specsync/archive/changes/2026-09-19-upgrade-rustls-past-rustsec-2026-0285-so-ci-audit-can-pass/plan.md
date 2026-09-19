---
change: upgrade-rustls-past-rustsec-2026-0285-so-ci-audit-can-pass
artifact: plan
---

# Plan

One file changes: `Cargo.lock`.

1. `cargo update -p rustls` on this branch, which resolves rustls 0.23.38 →
   0.23.45 and rustls-webpki 0.103.13 → 0.103.15.
2. Confirm `cargo audit` exits 0 aside from the already-allowed `instant`
   unmaintained warning.
3. No `Cargo.toml` edit: rustls is transitive via `ureq`.
