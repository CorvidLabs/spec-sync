---
spec: cmd_compact.spec.md
---

## Tasks

## Post-5.0 Test Debt

- [ ] Add an end-to-end CLI test that runs `compact --keep N` against a fixture spec and asserts the per-spec lines, summary, and `--dry-run` no-write behavior

## Done

- [x] Initial spec creation with all required sections
- [x] Requirements and acceptance criteria documented
- [x] Verified wrapper delegates to `compact::compact_changelogs` and matches the empty-result / dry-run / write paths
- [x] Confirmed the trimming logic is covered by `compact` inline tests (`test_compact_changelog`, `test_compact_no_change_needed`, `test_compact_three_column_table`)

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
