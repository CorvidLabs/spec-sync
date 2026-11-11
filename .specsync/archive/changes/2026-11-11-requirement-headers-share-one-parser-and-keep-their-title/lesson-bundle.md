# Lesson bundle: requirement-headers-share-one-parser-and-keep-their-title

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Requirement headers share one parser and keep their title
- **Kind**: BugFix
- **Specs**: change, validator
- **Paths**: src/change/collect_requirement_ids.rs, src/change_tests.rs, src/change_tests/titled_requirement_header_delta.rs, src/validator/requirement_ids.rs
- **Acceptance**: a MODIFIED delta applies to every header form the requirement parser accepts (multi-segment ids, punctuation, up to 3-space indent); the matched header line is written back unchanged

## Evidence

- Verification commit: `5659794e9d8b3caf2fadef3be123f57854c1e7d8`
- Base commit: `35e6f47ba4a12ba673230127905b3bc834445deb`
- Verified by: `specsync check --spec change --spec validator`

## From the change's context.md

# Context

Round 2 of #1024 (PR #1039), from t384's independent review. Round 1 hand-
rolled an id-shape guard narrower than the real parsers: `REQ-cmd-change-012`
(multi-segment module) and `REQ-x-001: Title` (punctuation after the id) were
still refused. Leif's call via t370: use the same id rule as
`is_requirement_id`, and keep the title — #1024 asked for it to be kept.

The fix: `requirement_heading_id` (validator) becomes `pub(crate)` and the
materializer's matcher delegates to it for `### ` targets; `## ` section keys
keep exact matching. On write, the matched header line is written back
unchanged, so the title survives; convergence comes from the tolerant matcher,
not from normalization. REQ-change-150's wording is updated by the delta.

## From the change's testing.md

# Testing

Unit tests in change_tests/titled_requirement_header_delta.rs covering the
reviewer's edge forms; the validator parser's own tests unchanged.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-150 | change_tests titled-header tests (updated) |
| REQ-validator-047 | change_tests titled-header tests exercising parser forms |

## Where these lessons go

- `specs/change/change.3md`
- `specs/validator/validator.3md`
