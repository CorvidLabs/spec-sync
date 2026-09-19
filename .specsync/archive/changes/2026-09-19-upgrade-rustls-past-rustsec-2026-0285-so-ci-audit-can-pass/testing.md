---
change: upgrade-rustls-past-rustsec-2026-0285-so-ci-audit-can-pass
artifact: testing
---

# Testing

Verified before review:

- `cargo update -p rustls` produced rustls 0.23.45 and rustls-webpki 0.103.15
  in `Cargo.lock`.
- `cargo audit` exits 0. The only remaining finding is the allowed
  `instant` 0.1.13 unmaintained warning (RUSTSEC-2024-0384).
- RUSTSEC-2026-0285 no longer appears.

Verified by CI on this pull request:

- The `audit` job succeeds on the full run, so `implementation-gate` and
  `Required CI gate` are no longer red solely because of rustls.
