# Lesson bundle: delta-materializer-matches-and-keeps-titled-requirement-headers

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Delta materializer matches and keeps titled requirement headers
- **Kind**: BugFix
- **Specs**: change, validator
- **Paths**: src/change/collect_requirement_ids.rs, src/validator/requirement_ids.rs, src/change_tests.rs, src/change_tests/titled_requirement_header_delta.rs
- **Acceptance**: a MODIFIED delta applies to every header form the requirement parser accepts (title suffix, punctuation, multi-segment ids, up to 3-space indent); the matched header line is written back unchanged; a following indented requirement block survives

## Evidence

- Verification commit: `b769930f41e072b559c04783824f229575e0e957`
- Base commit: `46f55652649a72ddf5e1715938b504b96686566b`
- Verified by: `specsync check --spec change --spec validator`

## From the change's context.md

# Context

Issue #1024, re-cut onto a fresh id per t370's numbering allocation (#1026
keeps REQ-change-150; this fix takes REQ-change-152). Carries the three rounds
of PR #1039's work: the materializer matches requirement blocks through the
shared `requirement_heading_id` parser (t384's round-2 review), the block-end
scan shares the parser's ATX indent rule (t384's round-3 finding: an indented
header's block ran through the next indented requirement), and a matched
header line is written back unchanged — the title is kept (coordinator
direction, matching what #1024 asked).

Supersedes #1039, whose branch had the id collision with #1026 baked into
archived records. READY track: the requirement text is Leif's at merge.

## From the change's testing.md

# Testing

change_tests/titled_requirement_header_delta.rs: eleven tests covering every
header form and boundary from #1039's three review rounds.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-152 | change_tests titled-header tests |
| REQ-validator-047 | same suite through the shared parser |

## How the titled-header tests run

`change check` executes only the in-process spec↔code sync (recorded as
`specsync check --spec change --spec validator`); `.specsync/sdd.json`
verification commands are not executed by the lifecycle. The titled-header
suite therefore runs two other ways, and both run again at the post-approval
re-verify:

- Locally at the final head: `cargo test titled_requirement_header`.
- In hosted CI: the `test` job runs the full suite at the final head.

## Where these lessons go

- `specs/change/change.3md`
- `specs/validator/validator.3md`
