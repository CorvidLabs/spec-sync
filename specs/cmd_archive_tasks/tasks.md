---
spec: cmd_archive_tasks.spec.md
---

## Tasks

(none open)

## Done

- [x] Initial spec creation with all required sections
- [x] Requirements and acceptance criteria documented
- [x] Verified wrapper delegates to `archive::archive_tasks` and matches the empty-result / dry-run / write paths
- [x] Confirmed delegate logic is covered by `archive` inline tests (`test_archive_completed_tasks`, `test_archive_no_completed`, `test_archive_preserves_existing`)
- [x] Add text, JSON/`--json`, and Markdown end-to-end dry-run coverage
- [x] Emit parse-clean structured output with explicit `would_change` and `applied` truth
- [x] Normalize JSON and Markdown paths to portable separators and cover Windows-style inputs
- [x] Render typed planned, succeeded, rolled-back, and failed operation collections
- [x] Exit 1 after rendering incomplete operations without claiming `applied: true`
- [x] Preserve literal Unix backslashes while normalizing Windows separators
- [x] Harden Markdown/GitHub paths against pipes, backtick runs, controls, and bidirectional injection
- [x] Replace task(s)/file(s) placeholders with truthful singular/plural labels
- [x] Sanitize text paths and errors against control and bidirectional terminal injection
- [x] Add explicit GitHub-format and structured zero-write failure integration coverage
- [x] Preserve legal Unix backslashes through Markdown/GitHub code-span rendering

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
