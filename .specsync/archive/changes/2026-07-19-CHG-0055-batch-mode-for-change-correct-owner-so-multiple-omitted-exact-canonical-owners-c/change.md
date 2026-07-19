---
id: CHG-0055-batch-mode-for-change-correct-owner-so-multiple-omitted-exact-canonical-owners-c
state: archived
type: bug_fix
base_commit: 37120cb60407efed08b4868858e76fb847d1ee9d
---

# Batch mode for change correct-owner so multiple omitted exact canonical owners can be audited and appended in one transactional correction before a single reapprove-verify-accept cycle

## Intent

Batch mode for change correct-owner so multiple omitted exact canonical owners can be audited and appended in one transactional correction before a single reapprove-verify-accept cycle

## Affected Canonical Specs

- `change`
- `cli_args`
- `cmd_change`

## Acceptance Criteria

- Repeated --path/--spec pairs, an optional JSON or TSV manifest, or --all-missing with one --spec append every validated exact owner correction in one transactional write; each correction remains an independent sequenced audit entry; any invalid entry fails closed with zero mutations; single-path correct-owner remains supported; unit and integration tests cover batch success, partial-failure atomicity, and --all-missing discovery.

## No-spec Rationale

Not applicable
