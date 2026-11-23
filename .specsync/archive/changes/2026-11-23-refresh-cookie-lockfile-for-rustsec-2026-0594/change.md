---
id: refresh-cookie-lockfile-for-rustsec-2026-0594
state: archived
type: bug_fix
base_commit: 71f1867d7ddcf2b075b49c421d2f87ea0b760be1
---

# Refresh cookie lockfile for RUSTSEC-2026-0594

## Intent

Refresh cookie lockfile for RUSTSEC-2026-0594

## Affected Canonical Specs

- None

## Acceptance Criteria

- Cargo.lock resolves cookie 0.18.3 instead of 0.18.1. Only the version and checksum in the existing cookie entry change.
- The resolved features and dependency graph match the baseline apart from the cookie version, and audit runs with the existing policy and clears RUSTSEC-2026-0594.
- The full project verification lane passes on the pinned Rust toolchain.

## No-spec Rationale

Resolve a compatible transitive dependency patch through Cargo.lock without changing the SpecSync contract.
