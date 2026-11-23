---
id: synchronize-change-summary-state-after-verification
state: verifying
type: bug_fix
base_commit: ba729a08dde0b269518fb9a5b9037155d1febcd3
---

# Synchronize change summary state after verification

## Intent

Synchronize change summary state after verification

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- After change check records either a passing verification or a completed failing verification, state.json, change.md, and change status all report verifying.
- Rechecking an active change whose summary still says implementing repairs its summary and keeps its Markdown notes.
- A pre-verification refusal preserves the prior lifecycle state, and a failed publication leaves the previous state and evidence together.
- change check --commit records the synchronized summary and leaves passing evidence usable for review.

## No-spec Rationale

Not applicable
