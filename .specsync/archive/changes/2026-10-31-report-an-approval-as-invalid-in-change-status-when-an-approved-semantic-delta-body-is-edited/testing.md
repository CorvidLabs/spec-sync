---
change: report-an-approval-as-invalid-in-change-status-when-an-approved-semantic-delta-body-is-edited
artifact: testing
---

# Testing

Each new test carries an honest label. A DISCRIMINATOR must fail on origin/main f240d37 and pass
with the fix. A CONTROL passes on both. The label is confirmed by running the new tests against a
build of unfixed f240d37 from a separate checkout, not by disabling the fix in place.

## Unit tests

These go in `src/change_tests/semantic_application_resolves_registry_backed_canonical_paths.rs`.
They reuse `change_with_an_approved_delta`, `SWAPPED_DELTA_BODY`, `strip_recorded_delta_digests`
and `approved_delta_body_as_a_crlf_checkout_writes_it`.

DISCRIMINATORS:

- `status_reports_an_edited_approved_delta_as_an_invalid_approval`: approved state, delta body
  edited. Asserts:
  - `summarize_change` returns `approval_valid == false`,
  - `next_action` names `change approve`,
  - `changed_approved_deltas == ["auth"]`,
  - `handoff.readiness == NotYet`,
  - `materialize_change_deltas` refuses the same tree, so status and check agree.
- `status_reports_an_edited_approved_delta_while_implementing_and_verifying`: the same assertions
  after `start_implementation`, and after a passing `check_change` moves the change to verifying.
- `handoff_summary_is_not_yet_for_an_edited_approved_delta`: the text-path `handoff_summary`
  equals `summary.handoff` and is `NotYet`.
- `audit_reports_an_edited_approved_delta`: `audit_project` reports, for this change, the
  "changed after approval" refusal `check_change` returns.
- `status_reports_a_withdrawn_delta_binding_as_an_invalid_approval`: uses the ledger shape of
  `a_later_definition_approval_may_not_withdraw_a_recorded_delta_binding`.

CONTROLS:

- `restoring_or_reapproving_an_edited_delta_restores_approval_validity`.
- `a_crlf_checkout_of_an_approved_delta_keeps_status_approval_valid`.
- `status_keeps_approval_valid_for_a_ledger_without_delta_digests`: strips digests, edits the
  delta, and expects `approval_valid` true, because materialization proceeds there too.
- `editing_the_plan_after_approval_keeps_approval_valid`: `plan.md` edited; `approval_valid` stays
  true and `check_change` passes.
- `changed_approved_deltas_is_omitted_when_empty`: serialized summary of a clean approved change
  has no such key.
- `finalized_change_summary_keeps_its_approval_and_terminal_verdicts`: archived fixture; summary
  `approval_valid` and `terminal_evidence` equal what f240d37 computes.

Existing handoff tests stay green unchanged, including
`handoff_stale_approval_is_not_yet_in_every_approved_state` and
`change_summary_carries_the_same_handoff_the_domain_computes`.

## Command tests

These go in `src/commands/change.rs`, `mod tests`, using `approved_change_fixture`.

- DISCRIMINATOR `text_next_action_names_reapproval_when_an_approved_delta_was_edited`.
- DISCRIMINATOR `text_next_action_names_reapproval_when_the_approved_scope_changed`, for the
  pre-existing stale-scope case.
- CONTROL `text_next_action_names_check_for_a_current_approval`.

## Integration test

This goes in `tests/integration/change.rs`, through the built binary, following
`status_prints_a_handoff_line_and_json_carries_it`.

`status_json_and_text_report_an_edited_approved_delta`:

1. Create, answer and approve a change with a fixture actor in a temp Git repo.
2. Append a requirement block to its delta.
3. Assert:
   - `change status --json` has `summary.approval_valid == false`, a `change approve` next action,
     `summary.changed_approved_deltas == ["auth"]` and `summary.handoff.readiness == "not-yet"`,
   - text status prints a `change approve` `Next:` line and a `not yet` `Handoff:` line,
   - `change check` and `change audit` both exit 1.
4. Restore the delta. Status is valid again with a `change check` next action.

## Regression lanes

- `cargo test change::` (the component route for `change`).
- `cargo test commands::change::`.
- `cargo test --test integration change::`.
- `fledge lanes run verify` (fmt, clippy, types, full test, release build, strict specs).
- `specsync check --strict --require-coverage 100`.
- `specsync change audit`.
- `fledge lanes run pre-push` before every push.
- `fledge trust verify` before calling the work complete.

## Requirement evidence

| Requirement | Required evidence |
|-------------|-------------------|
| REQ-change-111 | Discriminators: `status_reports_an_edited_approved_delta_as_an_invalid_approval`, `status_reports_an_edited_approved_delta_while_implementing_and_verifying`, `handoff_summary_is_not_yet_for_an_edited_approved_delta`, `audit_reports_an_edited_approved_delta`, `status_reports_a_withdrawn_delta_binding_as_an_invalid_approval`. Controls: CRLF-only, restored or re-approved body, ledger without digests, `plan.md` edit, omitted empty list, finalized-change summary. Integration: `status_json_and_text_report_an_edited_approved_delta`. |
| REQ-cmd-change-005 | `text_next_action_names_reapproval_when_an_approved_delta_was_edited`, `text_next_action_names_reapproval_when_the_approved_scope_changed`, `text_next_action_names_check_for_a_current_approval`, and the text half of `status_json_and_text_report_an_edited_approved_delta`. Existing REQ-cmd-change-005 coverage keeps passing. |

No result is claimed here. Observed results are recorded after the tests are written and run.

## Observed results (2026-10-31)

Labels were confirmed against an export of f240d37 with only the new tests applied (no fix):

- Failed on f240d37, as labelled DISCRIMINATOR: `status_reports_an_edited_approved_delta_as_an_invalid_approval`, `status_reports_an_edited_approved_delta_while_implementing_and_verifying`, `handoff_summary_is_not_yet_for_an_edited_approved_delta`, `audit_reports_an_edited_approved_delta`, `status_reports_a_withdrawn_delta_binding_as_an_invalid_approval`, `text_next_action_names_reapproval_when_an_approved_delta_was_edited`, `text_next_action_names_reapproval_when_the_approved_scope_changed`, and the integration test `status_json_and_text_report_an_edited_approved_delta` (status reported `approval_valid: true`).
- Passed on f240d37, as labelled CONTROL: `restoring_or_reapproving_an_edited_delta_restores_approval_validity`, `a_crlf_checkout_of_an_approved_delta_keeps_status_approval_valid`, `status_keeps_approval_valid_for_a_ledger_without_delta_digests`, `editing_the_plan_after_approval_keeps_approval_valid`, `changed_approved_deltas_is_omitted_when_empty`, `finalized_change_summary_keeps_its_approval_and_terminal_verdicts` and `text_next_action_names_check_for_a_current_approval`. The refactored `a_later_definition_approval_may_not_withdraw_a_recorded_delta_binding` also still passes there.
- With the fix, all of them pass. The full `cargo test` run passes: 2,614 unit tests and 463 integration tests.

Deviation: the integration fixture uses a `"version": 2` policy. Under a version-1 policy `change new` writes a workflow-v1 record, and the test then passed on f240d37 for the wrong reason. The appended requirement carries Acceptance Criteria so that `change check` refuses at the delta binding and not in `validate_definition`.
