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
- [x] Make generated summary folding byte-for-byte idempotent and preserve the original count/range
- [x] Preserve escaped pipes, exact table width, and trailing-newline state
- [x] Prevent user-authored `Compacted:` prose from being mistaken for generated state
- [x] Add focused unit and end-to-end CLI regressions for issue #417
- [x] Complete issue #417 structured command output in the owning `cmd_compact`, `cmd_archive_tasks`, and CLI modules
- [x] Add provenance-bound summary ownership and reject duplicate generated summaries
- [x] Preserve CRLF/mixed line endings and parse escaped/code-span pipes with correct backslash parity
- [x] Add checked fixed-width summary counts, keep-zero coverage, and first-contiguous-table isolation
- [x] Preflight and stage compact replacements before publication with typed incomplete/partial reporting
- [x] Ignore fenced/indented changelog examples and prefix headings such as `## Change Logger`
- [x] Preserve a missing final newline for LF and CRLF files when `keep = 0`
- [x] Retain complete planned counts on staging failure and characterize deterministic late-publish partial results

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
