---
change: report-an-approval-as-invalid-in-change-status-when-an-approved-semantic-delta-body-is-edited
artifact: docs
---

# Docs

`site/src/content/docs/cli.md` gets one paragraph, in the existing `change status` paragraph that
already documents `summary.handoff`. It says:

- `summary.approval_valid` is false whenever a gate that needs the approval would refuse: an
  approved scope that changed, or a semantic delta body edited after approval.
- The edited modules are listed in `summary.changed_approved_deltas`.
- Editing `plan.md` or another execution artifact after approval stales verification, not
  approval.

Two other doc changes:

- After verification, the change and cmd_change lessons
  (`specs/change/change.3md`, `specs/cmd_change/cmd_change.3md`) record that status had dropped
  the delta binding. They also record that `text_mode_next_action` never consulted approval
  currency.
- The canonical requirement texts arrive through the approved deltas.

`CHANGELOG.md` is written by the release change, not here. The PR body states the observable JSON
change and the new `change audit` failure so the release notes can carry them.
