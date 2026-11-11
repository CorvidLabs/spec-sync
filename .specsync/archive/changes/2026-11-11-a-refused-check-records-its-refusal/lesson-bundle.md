# Lesson bundle: a-refused-check-records-its-refusal

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: A refused check records its refusal
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/sdd_version.rs, src/change/validate_loaded_change.rs, src/change/historical_definition_digest_matches.rs, src/change_tests.rs, src/change_tests/, tests/
- **Acceptance**: a change check refused before verification appends a passed:false attempt with the reason to verification-attempts.json; verification.json is untouched

## Evidence

- Verification commit: `9585be5208336c50ce6986a545a10593f2ebca3e`
- Base commit: `46f55652649a72ddf5e1715938b504b96686566b`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Issue #847: when `change check` refuses before verification starts (open
tasks, missing requirement evidence, an unattributed gate refusal), it exits 1
and records nothing — no `verification-attempts.json` entry, no reason
anywhere in the workspace. A reader (Chough's `next`, a later session) cannot
tell the check was tried or why it failed.

The comment at the gate says the early refusal is deliberate, but the reason
it gives — "refused before any attempt is recorded" — is the defect. The fix
appends a `passed: false` attempt carrying the refusal reason to
`verification-attempts.json` only: `verification.json` (the last completed
verification) is never overwritten by a refusal. The attempt gains an optional
`refusal` field; absent in every prior record, so old bytes stay valid.

## From the change's testing.md

# Testing

Integration: an approved change with an open task gets checked; the refusal
exits 1, and verification-attempts.json holds a passed:false entry naming the
reason while no verification.json appears.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-154 | integration test driving the tasks-gate refusal |

## Where these lessons go

- `specs/change/change.3md`
