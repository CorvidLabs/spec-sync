---
spec: cmd_compact.spec.md
---

## Tasks

(none open)

## Done

- [x] Initial spec creation with all required sections
- [x] Requirements and acceptance criteria documented
- [x] Verified wrapper delegates to `compact::compact_changelogs` and matches the empty-result / dry-run / write paths
- [x] Confirmed the trimming logic is covered by `compact` inline tests (`test_compact_changelog`, `test_compact_no_change_needed`, `test_compact_three_column_table`)
- [x] Add an end-to-end CLI test that runs `compact --keep N` against a fixture spec and asserts the per-spec lines, summary, `--dry-run` no-write behavior, and byte-identical second run
- [x] Add parse-clean JSON/`--json` equivalence and Markdown/GitHub output with truthful dry-run fields
- [x] Add focused text, JSON, and Markdown CLI integration coverage for issue #417
- [x] Normalize JSON and Markdown paths to portable separators and cover Windows-style inputs
- [x] Preserve literal Unix backslashes and render adversarial Markdown/GitHub paths safely
- [x] Render typed complete/partial failures and exit nonzero without false `applied` success
- [x] Use correct singular/plural labels for aggregate spec counts
- [x] Preserve literal Unix backslashes through Markdown/GitHub rendering
- [x] Prove late-publish partial counts and errors in parseable JSON and Markdown

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
