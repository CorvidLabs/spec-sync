# Lesson bundle: close-fenced-delta-replay-review-findings

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Close fenced delta replay review findings
- **Kind**: BugFix
- **Specs**: change, validator
- **Paths**: src/change/collect_requirement_ids.rs, src/change_tests/fenced_requirement_example_delta.rs, src/validator/requirement_ids.rs, specs/validator/validator.spec.md, specs/change/change.3md, specs/change/tasks.md, specs/change/testing.md
- **Acceptance**: The existing requirement reader and the canonical materializer apply the same fence rules. A malformed backtick opener leaves both requirement and spec section headings visible. Valid fenced examples leave the intended block boundaries intact.
- **Acceptance**: Already-applied checks accept a removal when only a fenced example remains and a modification when the canonical block already carries the new body.

## Evidence

- Verification commit: `f036322e293118ef2b02f04cf86293ba73bc2ff2`
- Base commit: `4df769d3130d17c39ead061077400b784bce2d13`
- Verified by: `specsync check --spec change --spec validator`

## From the change's context.md

# Context

The review of PR #1144 found that canonical-block matching called the general spec parser's fence blanker, while the requirement reader has its own CommonMark fence recognizer. A backtick in a proposed opener's info string therefore hides a real heading from materialization while the requirement reader sees it. The review also asks for regression coverage of the shared matcher's already-applied removal and modification paths.

This follow-up adds three tests and uses the existing validator helper. It keeps the original archive and review evidence on the PR.

## From the change's design.md

# Design

Make the requirement reader's fence recognizer crate-visible and document that internal interface in the validator module spec. The helper's body and reader call sites retain their behavior.

The canonical block matcher scans the original Markdown, tracking byte offsets and applying the helper before matching a heading or deciding the next block boundary. That single scan serves both requirement and spec section targets. The general parser blanker keeps its own existing callers.

## From the change's testing.md

# Testing

- The replay tests exercise removals with only a fenced example remaining and modifications already reflected by the real requirement.
- The malformed-opener regression uses both requirement and spec section targets and verifies the resulting Markdown. The pre-fix matching path hides the intended requirement.
- Run the focused fence and Markdown block tests and the full Fledge verification lane. Verify strict SpecSync validation and audit, coverage, pre-push and Trust on the exact implementation tip.

## Where these lessons go

- `specs/change/change.3md`
- `specs/validator/validator.3md`
