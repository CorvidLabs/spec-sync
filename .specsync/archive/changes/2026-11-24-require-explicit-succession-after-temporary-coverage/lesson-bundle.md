# Lesson bundle: require-explicit-succession-after-temporary-coverage

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Require explicit succession after temporary coverage
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/historical_definition_digest_matches.rs, src/change/consider_accepted_evidence_anchor.rs, src/change/validate_loaded_change.rs, src/change_tests/provisional_coverage.rs, specs/change/, site/src/content/docs/workflow.md
- **Acceptance**: After a temporary coverer is accepted, an active predecessor gets successor coverage only through an explicit valid succession obligation. On an archived predecessor, status, show and archive audit continue to warn for a file when the accepted or archived coverer lacks that obligation; a valid matching succession clears the warning and archive integrity remains independent.

## Evidence

- Verification commit: `8c4421a3bbf4655745dc2621514e5b6cb2858451`
- Base commit: `6e5a63b3d9d83151dde78734b4557a577d1a494b`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

0xLeif clarified the intended transition during review of #1132. REQ-change-014 requires explicit semantic succession. REQ-change-167 described a state-only promotion for an in-flight coverer after acceptance.

## From the change's design.md

# Design

Use the existing signed succession rules for the recorded covering change and the specific predecessor input. Apply the result only to the display warning returned by status, show and archive audit. An archive keeps its historical evidence classification.

## From the change's testing.md

# Testing

Exercise a plain scoped coverer and a declared successor for an active predecessor and for an archive. Check both accepted and archived covering states, the displayed warning, the audit outcome and the archived predecessor’s evidence.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-167 | Archive and active predecessor regressions in `src/change_tests/provisional_coverage.rs`, plus SpecSync structural check |

## Where these lessons go

- `specs/change/change.3md`
