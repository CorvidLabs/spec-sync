---
id: refresh-url-lockfile-for-rustsec-2026-0592
state: implementing
type: bug_fix
base_commit: ba729a08dde0b269518fb9a5b9037155d1febcd3
---

# Refresh url lockfile for RUSTSEC-2026-0592

## Intent

Refresh url lockfile for RUSTSEC-2026-0592

## Affected Canonical Specs

- None

## Acceptance Criteria

- Cargo.lock resolves url 2.5.9 and its required form_urlencoded 1.2.3 with no other dependency changes.
- Cargo audit clears RUSTSEC-2026-0592 and the full project verification lane passes on pinned Rust.

## No-spec Rationale

A compatible dependency patch and its required companion update change the lockfile without changing the SpecSync contract.
