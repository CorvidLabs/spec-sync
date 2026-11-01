---
id: report-an-approval-as-invalid-in-change-status-when-an-approved-semantic-delta-body-is-edited
state: archived
type: bug_fix
base_commit: f240d379b9f023f5766e5c9b39508eb84e03e2bb
---

# Report an approval as invalid in change status when an approved semantic delta body is edited

## Intent

Report an approval as invalid in change status when an approved semantic delta body is edited

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- After a workflow-v2 change is approved by a definition approval that recorded semantic delta digests, editing an approved delta body (committed or not) makes `specsync change status <id> --json` report `summary.approval_valid: false`, `summary.next_action` naming `specsync change approve <id> --actor <name>`, and `summary.handoff.readiness` `not-yet` in the approved, implementing and verifying states, the same verdict `specsync change check <id>` reaches when it refuses that delta.
- That summary lists the edited modules, sorted, in a new `changed_approved_deltas` array that is omitted when empty, so no other status payload gains a key.
- Text `specsync change status <id>` prints the same re-approval `Next:` action and a `not yet` `Handoff:` line, and `specsync change audit` reports the edited delta as an error for that active change. Text `Next:` names re-approval for every approved, implementing or verifying change whose approval is not current, including the existing stale-scope case.
- Restoring the approved delta bytes, or re-approving, returns status to `approval_valid: true` and `run specsync change check <id>`. Re-encoding an unedited delta to CRLF does not invalidate the approval. A later definition approval that withdraws a recorded delta binding also reports `approval_valid: false`.
- Unchanged: a ledger that never recorded delta digests (every pre-binding approval and every archive) keeps today's status; draft, accepted and archived summaries keep today's `approval_valid` and terminal-evidence verdicts; historical and archived validation, digests and persisted ledgers are not altered; editing `plan.md` or another artifact outside the approved scope keeps `approval_valid: true` and only stales execution evidence.
- Regression tests fail on origin/main f240d37 and pass with the fix: a discriminator for each state above, labelled controls for every unchanged case, and an integration test that drives `change status --json` and text status through the built binary.

## No-spec Rationale

Not applicable
