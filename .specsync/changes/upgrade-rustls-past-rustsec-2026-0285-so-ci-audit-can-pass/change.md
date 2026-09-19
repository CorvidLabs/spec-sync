---
id: upgrade-rustls-past-rustsec-2026-0285-so-ci-audit-can-pass
state: implementing
type: operations
base_commit: 65c51fe76f7c2ca34ab94febfc7e55526c242f02
---

# Upgrade rustls past RUSTSEC-2026-0285 so CI audit can pass

## Intent

Upgrade rustls past RUSTSEC-2026-0285 so CI audit can pass

## Affected Canonical Specs

- None

## Acceptance Criteria

- cargo audit exits 0 because Cargo.lock pins rustls at >=0.23.45, clearing RUSTSEC-2026-0285. No other lockfile or manifest change is required.

## No-spec Rationale

Transitive rustls bump in Cargo.lock only; no canonical spec behaviour changes.
