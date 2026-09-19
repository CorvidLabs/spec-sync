# Lesson bundle — upgrade-rustls-past-rustsec-2026-0285-so-ci-audit-can-pass

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Upgrade rustls past RUSTSEC-2026-0285 so CI audit can pass
- **Kind**: Operations
- **Paths**: Cargo.lock
- **Acceptance**: cargo audit exits 0 because Cargo.lock pins rustls at >=0.23.45, clearing RUSTSEC-2026-0285. No other lockfile or manifest change is required.

## Evidence

- Verification commit: `84baa9d4dd9a461807d05f44ec525410995dbb83`
- Base commit: `65c51fe76f7c2ca34ab94febfc7e55526c242f02`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

PR #788 only adds a Human intent block to `AGENTS.md`. That path is not
meaningful, so the lifecycle gate was green, but `classify` still selected a
full CI run. `cargo audit` then failed on rustls 0.23.38,
RUSTSEC-2026-0285 (TLS 1.3 handshake messages accepted across encryption
level boundaries; fix is >=0.23.45, dated 2026-09-14). Main last went green
on 2026-09-12, before the advisory.

`Cargo.lock` is a meaningful path, so this bump needs its own change record.
Do not ignore the advisory. The remaining `instant` unmaintained warning is
already allowed.

spec-sync does not depend on rustls directly; `ureq` does. `cargo update -p rustls`
also moved `rustls-webpki` 0.103.13 → 0.103.15.

## From the change's testing.md

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

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
