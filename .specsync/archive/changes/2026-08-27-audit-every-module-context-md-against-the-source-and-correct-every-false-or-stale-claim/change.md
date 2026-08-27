---
id: audit-every-module-context-md-against-the-source-and-correct-every-false-or-stale-claim
state: archived
type: documentation
base_commit: dccd82105956d62df76bb4fec9fb777c4b31f15b
---

# Audit every module context.md against the source and correct every false or stale claim

## Intent

Audit every module context.md against the source and correct every false or stale claim

## Affected Canonical Specs

- `agents`
- `change`
- `changelog`
- `cmd_agents`
- `cmd_check`
- `cmd_comment`
- `cmd_coverage`
- `cmd_deps`
- `cmd_diff`
- `cmd_hooks`
- `cmd_init`
- `cmd_init_registry`
- `cmd_new`
- `cmd_report`
- `cmd_scaffold`
- `cmd_score`
- `cmd_wizard`
- `commands`
- `comment`
- `deps`
- `exports`
- `git_utils`
- `github`
- `hooks`
- `ignore`
- `importer`
- `manifest`
- `mcp`
- `merge`
- `output`
- `parser`
- `registry`
- `schema`
- `validator`

## Acceptance Criteria

- No specs/<module>/context.md asserts a symbol, file path, or test name that does not exist in the tree: check_project_quiet, auto_regen_stale_specs, remove_section, src/exports.rs, build_schema in validator.rs, and the deleted CI lifecycle workflows are all gone or restated as history.
- Every count a context.md states is the number a stated command produces today: tracked .md files, archived approval ledgers, unit and integration test totals, exports source files, and the cmd_coverage / cmd_diff / cmd_report integration-test counts.
- No context.md claims a behaviour the code contradicts: the change-sequence ledger is described as written by floor_sequence_ledger_to_committed rather than as read-only, deps records rather than silently swallows unreadable input, registry parses TOML with the toml crate, output is not claimed to do no file I/O, and coverage reports a zero denominator as null rather than 100%.
- Claims that were true only before a later change are marked as history rather than left in the present tense: parser.rs handling CRLF, CHG-0063 and CHG-0066 being under verification, and the CI lifecycle reimplementation deleted by #499.
- Judgement, rationale, and historical narrative are left untouched; only factual assertions about the current codebase are edited.

## No-spec Rationale

corrects false and stale factual claims in specs/<module>/context.md companions only; no canonical spec text, requirement, or behaviour changes
