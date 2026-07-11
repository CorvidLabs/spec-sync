---
spec: compact.spec.md
---

## Tasks

(none open)

## Done

- [x] `compact_changelogs` walks all specs via `find_spec_files`, writes unless `dry_run`
- [x] `compact_spec_changelog` locates the `## Change Log` section (next `## ` heading or EOF)
- [x] Preserve header + separator rows; keep last `keep` data rows
- [x] Build summary row with date range and `Compacted: N entries` count
- [x] 2-column and 3+ column table summary formats
- [x] `extract_first_cell` for date-range extraction
- [x] Filter out results where `removed == 0`
- [x] Bold-red error on write failure, continue with remaining specs
- [x] Populate requirements.md with user stories and acceptance criteria (2026-04-10)

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
