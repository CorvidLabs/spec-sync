---
id: CHG-0065-make-issue-417-changelog-compaction-idempotent-and-provide-truthful-portable-str
state: archived
type: bug_fix
base_commit: f74d52fdb6792dac2df89f545331b796dd7838ca
---

# Make issue 417 changelog compaction idempotent and provide truthful portable structured maintenance output

## Intent

Make issue 417 changelog compaction idempotent and provide truthful portable structured maintenance output

## Affected Canonical Specs

- `cli`
- `archive`
- `cmd_compact`
- `cmd_archive_tasks`
- `compact`

## Acceptance Criteria

- Repeated compact runs are byte-for-byte idempotent; only provenance-marked generated summaries are folded; ambiguous summaries, malformed tables, and count overflow fail closed; escaped/code-span pipes, exact LF/CRLF bytes, kept counts, and singular/plural output remain correct; compact and archive-tasks preflight and atomically publish planned replacements, report every incomplete/partial operation, emit parse-clean ANSI-free JSON and injection-safe Markdown/GitHub, preserve literal Unix backslashes while normalizing Windows separators, and exit nonzero without false success on failure; targeted regressions, full repository verification, strict spec coverage and scoring, trust verification, independent review, private sandbox replay, and GitHub CI all pass.

## No-spec Rationale

Not applicable
