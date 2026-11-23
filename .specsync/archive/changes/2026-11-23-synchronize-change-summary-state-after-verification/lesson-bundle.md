# Lesson bundle: synchronize-change-summary-state-after-verification

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Synchronize change summary state after verification
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/sdd_version.rs, src/change/validate_loaded_change.rs, src/change_tests/semantic_application_resolves_registry_backed_canonical_paths.rs, tests/integration/change.rs, site/src/content/docs/workflow.md, specs/change/change.3md
- **Acceptance**: After change check records either a passing verification or a completed failing verification, state.json, change.md, and change status all report verifying.
- **Acceptance**: Rechecking an active change whose summary still says implementing repairs its summary and keeps its Markdown notes.
- **Acceptance**: A pre-verification refusal preserves the prior lifecycle state, and a failed publication leaves the previous state and evidence together.
- **Acceptance**: change check --commit records the synchronized summary and leaves passing evidence usable for review.

## Evidence

- Verification commit: `1b46ce93b09634088f5124a11e10be082a6ed4cc`
- Base commit: `ba729a08dde0b269518fb9a5b9037155d1febcd3`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Issue #1143 reports that materialization writes implementing to state.json and change.md, then verification updates only state.json. The completed attempt already uses journaled publication, and the Markdown summary is outside the verification input digest. The workflow also directs people to put handoff notes in change.md.

## From the change's testing.md

# Testing

Use approved lifecycle fixtures to exercise first check, completed sync failure, legacy summary refresh, transaction rollback, and check --commit. Reuse the scoped review test after the commit gate.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-check-summary-state-001 | change unit tests exercise passing, failed, retried, and interrupted verification. The CLI integration test review_and_ship_refuse_a_verification_commit_that_lacks_the_work confirms the committed change.md state and records a scoped review after check --commit. |

## Where these lessons go

- `specs/change/change.3md`
