---
id: CHG-0100-fail-closed-in-text-lifecycle-views-when-a-correction-ledger-is-invalid
state: archived
type: bug_fix
base_commit: c6e76b34efd111b40a22ab16b6bc45be692dbe22
---

# Fail closed in text lifecycle views when a correction ledger is invalid

## Intent

Fail closed in text lifecycle views when a correction ledger is invalid

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- Text change status, show, and list SHALL fail with a generic correction-ledger integrity diagnostic when corrections.json is invalid; the diagnostic SHALL not disclose ledger content or digests, while valid ledgers retain their current text output.

## No-spec Rationale

Sandbox issue #17 proves that text status and show can report a healthy-looking correction count while the machine-readable view rejects an invalid correction ledger.
