---
change: CHG-0031-keep-the-rustsec-audit-job-compatible-with-the-rust-1-89-msrv
artifact: testing
---

# Testing

- Run `actionlint .github/workflows/ci.yml`.
- Run the configured SpecSync lifecycle verification.
- Push the accepted fix and require the hosted `audit` job to execute `cargo audit` successfully.
- Require the aggregate `Required CI gate` to pass without treating audit as optional.
