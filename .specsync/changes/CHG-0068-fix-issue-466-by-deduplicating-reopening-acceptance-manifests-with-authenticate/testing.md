---
change: CHG-0068-fix-issue-466-by-deduplicating-reopening-acceptance-manifests-with-authenticate
artifact: testing
---

# Testing

## Characterization

- Build a lifecycle change with at least 393 signed acceptance entries.
- Complete accepted → stale → reopen → verify → accept for A/B/A manifest states.
- Before the fix, assert each reopening embeds another full manifest in `approvals.json`.

## Targeted Regression Coverage

- New reopening events serialize without an embedded acceptance manifest.
- A/B/A history creates exactly two immutable objects and reuses A byte-for-byte.
- Ledger-size growth for another identical reopen is bounded independently of manifest entry count.
- Schema-v1 embedded events load, check, reopen again, reaccept, and archive.
- Schema-v2 events hydrate to the same prior `VerificationRecord` observed by lifecycle callers.
- Missing, symlinked, non-file, oversized, malformed, unknown-field, path/digest-mismatched, and
  verification-inconsistent objects fail closed before mutation or trust.
- Existing objects with identical content are reused; conflicting bytes at the same digest fail.
- Failed ledger publication never creates a trusted dangling reference.
- Active-to-dated-archive moves preserve object resolution and authenticated history.
- `migrate 5.0` repairs only eligible legacy digest fields, is idempotent, and leaves compact
  records byte-identical.

## Broader Gates

- Targeted Rust unit and CLI integration tests.
- Full unit, integration, release-build, docs, audit, Linux/macOS/Windows CI matrix.
- `specsync change check`, `specsync check --strict`, 100% spec coverage, and score at least 80.
- `fledge lanes run verify` and `fledge trust verify`.
- Augur must not block; Attest provenance is recorded only after verification passes.
- One independent reviewer checks every #466 acceptance row and another performs adversarial
  persistence, path, compatibility, and regression review.
