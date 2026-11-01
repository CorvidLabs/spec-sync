---
change: report-an-approval-as-invalid-in-change-status-when-an-approved-semantic-delta-body-is-edited
artifact: requirements
---

# Requirements

### REQ-change-111

The system SHALL report a definition approval as invalid in change status, in the handoff
verdict, and in the active-workspace audit whenever the semantic delta bodies that approval
recorded no longer match the change's delta files, using the same binding that materialization
and acceptance enforce. It SHALL NOT reinterpret an approval that recorded no delta binding.

Acceptance Criteria
- For an approved, implementing, or verifying change whose effective definition approval recorded
  delta digests, a delta body edited after approval has three effects:
  - `summary.approval_valid` is false.
  - `summary.next_action` names `specsync change approve <id> --actor <name>`.
  - The handoff readiness is `not-yet`.
- `summary.changed_approved_deltas` lists the edited modules in sorted order and is omitted from
  JSON when empty.
- `change audit` reports, for that change, the same refusal `change check` gives.
- The following keep `approval_valid` true or return it to true:
  - a body that differs only in CRLF line endings,
  - a restored body,
  - a re-approved body.
- A later definition approval that withdraws a recorded binding reads as invalid, as
  materialization already refuses it.
- A ledger in which no definition approval recorded a delta digest keeps its current status
  verdict.
- The following are unchanged:
  - draft, accepted, and archived summaries,
  - accepted and archived authentication,
  - persisted ledgers and every digest.
- An edit to `plan.md` or another artifact outside the approved scope keeps `approval_valid` true
  and stales only execution evidence.

### REQ-cmd-change-005 (modified)

The status bullet changes so that status requests renewed approval for either of two reasons:

- a material stable-scope change, or
- a semantic delta body edited after the approval that recorded it, naming each such module.

Other execution and evidence-only changes still direct the user to `change check`. Text and JSON
name the same re-approval action whenever the approval is not current. The full restated
requirement is in `deltas/cmd_change.md`.
