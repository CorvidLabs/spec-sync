---
change: CHG-0065-make-issue-417-changelog-compaction-idempotent-and-provide-truthful-portable-str
artifact: requirements
---

# Requirements

## REQ-417-001 — Idempotent compaction

Repeated `compact` runs SHALL be byte-for-byte stable when no new ordinary changelog rows exceed
the keep limit. A later run with new excess rows SHALL fold prior generated counts into one summary
and retain the original range start.

## REQ-417-002 — Exact row ownership

Only summary rows carrying the exact SpecSync provenance marker SHALL be treated as generated
state. Unmarked lookalikes SHALL remain ordinary history, and multiple marked summaries SHALL fail
closed. Odd-backslash escapes and Markdown code spans SHALL retain embedded pipes while even
backslash runs expose delimiters. Only the first contiguous width-valid table SHALL be compacted.

## REQ-417-003 — Byte and count fidelity

Compaction SHALL preserve every LF/CRLF terminator and all bytes outside the changed rows.
Fixed-width summary counts SHALL use checked arithmetic. Reported kept counts SHALL exclude the
generated summary row. Entry/spec labels SHALL use correct singular and plural forms.

## REQ-417-004 — Structured maintenance output

`compact` and `archive-tasks` SHALL honor text, JSON, Markdown, and GitHub output selection.
`--json` SHALL equal `--format json`. JSON SHALL be one parseable ANSI-free document. Markdown and
GitHub SHALL contain a structured result table and truthful summary.

## REQ-417-005 — Dry-run truth

Dry runs SHALL make no file writes. Structured output SHALL report selected work through
`would_change` while leaving `applied` false.

## REQ-417-006 — Cross-platform determinism

JSON and Markdown/GitHub repo-relative result paths SHALL use `/` separators on Linux, macOS, and
Windows without changing legal Unix literal backslashes. Markdown/GitHub paths SHALL use sanitized
variable-length code spans that cannot inject rows through pipes, backticks, controls, or bidi
formatting. Existing text output SHALL sanitize unsafe terminal characters.

## REQ-417-008 — Transactional maintenance truth

`compact` and `archive-tasks` SHALL inspect and stage every selected replacement before
publication. Preflight/staging failure SHALL write nothing. A later publication/rollback failure
SHALL be represented as incomplete/partial structured evidence, SHALL exit 1, and SHALL never claim
`applied: true` or complete success.

## REQ-417-007 — Delivery evidence

Targeted regressions, the full repository lane, strict 100% spec coverage, score thresholds, trust
verification, independent review, private-sandbox replay, and required GitHub CI SHALL pass before
closing approval.
