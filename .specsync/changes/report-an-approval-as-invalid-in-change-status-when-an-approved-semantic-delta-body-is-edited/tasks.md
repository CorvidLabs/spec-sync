---
change: report-an-approval-as-invalid-in-change-status-when-an-approved-semantic-delta-body-is-edited
artifact: tasks
---

# Tasks

- [ ] Add the discriminator and control tests in `testing.md` and record which fail on origin/main f240d37.
- [ ] Return the drifted module list from the delta-binding helper without changing its refusal text.
- [ ] Compose scope currency and delta-binding currency for approved, implementing and verifying summaries, and add `ChangeSummary.changed_approved_deltas` (omitted when empty).
- [ ] Use the composed predicate in `handoff_summary` and in the `change audit` active-record loop.
- [ ] Export `approval_current_for_guidance` and have text `Next:` name re-approval when it is false.
- [ ] Document the corrected `summary.approval_valid` and the new list in `site/src/content/docs/cli.md`.
- [ ] Update the change and cmd_change testing, tasks and lessons (`change.3md`, `cmd_change.3md`) after verification.
- [ ] Run the regression lanes in `testing.md` and record the observed results.

## Human gates

Scope approval of this change by a human is required before any implementation starts. No
approval is recorded by this draft.

Implementation review (`specsync change review <id> --reviewer <human>`) by a human follows a
current `change check`. Finalization, merge and any release are separate human decisions.

## After verification

Run `fledge lanes run pre-push` before every push and `fledge trust verify` before calling the
work complete. Hosted CI and Trust results for the exact head are recorded as evidence, not
assumed.
