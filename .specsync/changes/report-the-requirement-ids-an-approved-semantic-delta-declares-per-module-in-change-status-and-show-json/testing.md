---
change: report-the-requirement-ids-an-approved-semantic-delta-declares-per-module-in-change-status-and-show-json
artifact: testing
---

# Testing

Each test carries an honest label. A DISCRIMINATOR fails on origin/main a80748f and passes with
the change. A CONTROL passes on both, or pins behavior that must not move. Labels are confirmed by
running the new tests against a build of unfixed a80748f from a separate checkout, not by
disabling the change in place.

On a80748f no `approved_delta_summaries` exists, so every unit test that calls it fails to compile
there. To label them, apply the tests together with a stub
`approved_delta_summaries(..) -> Vec::new()` and the two types, and nothing else.

## Unit tests (new `src/change_tests/approved_delta_requirements.rs`)

Fixture: `change_with_an_approved_requirement_delta(root)`, built on
`completed_section_only_current_record` with an `auth` delta that has three items:

- `## ADDED` / `### REQUIREMENT REQ-auth-002`, which has SHALL and Acceptance Criteria and whose
  body mentions `REQ-auth-009` in prose,
- `## MODIFIED` / `### SPEC SECTION Purpose`,
- `## REMOVED` / `### SPEC SECTION Legacy Notes`.

The fixture writes requirement evidence for REQ-auth-002 into `testing.md`, then
`approve_definition`.

DISCRIMINATORS:

- `approved_delta_summaries_report_declared_requirements_and_sections`: approved state. The
  `auth` entry has `delta_intact == Some(true)`, `requirements.added == ["REQ-auth-002"]`,
  empty `modified` and `removed` requirements, `sections.modified == ["Purpose"]`,
  `sections.removed == ["Legacy Notes"]` and `materialized == Some(false)`.
- `a_requirement_mentioned_in_prose_is_not_reported`: `REQ-auth-009` is in no list.
- `an_edited_approved_delta_reports_no_approved_requirements`: body edited after approval.
  - The entry has `delta_intact == Some(false)` and `requirements`, `sections` and
    `materialized` all `None`.
  - The serialized entry has none of those keys.
  - `summarize_change` gives `changed_approved_deltas == ["auth"]` and `!approval_valid`.
- `a_withdrawn_delta_binding_reports_every_module_not_intact`: uses the ledger shape of
  `a_later_definition_approval_may_not_withdraw_a_recorded_delta_binding`.
- `a_ledger_without_delta_digests_reports_unknown_integrity`: `strip_recorded_delta_digests`.
  - `delta_intact == None`, serialized as `null`, with no lists.
  - It is not `Some(false)`, even after the body is edited.
- `materialized_turns_true_after_check_and_false_after_a_reapproved_correction`:
  - After `materialize_change_deltas`, `materialized == Some(true)`.
  - On `change_materialized_then_corrected_and_re_approved`, `materialized == Some(false)` and
    the `requirements` and `sections` are those of the corrected body.
  - After a second `materialize_change_deltas`, it is `Some(true)` again.
- `approved_delta_summaries_hold_in_implementing_and_verifying`: the same entry after
  `start_implementation`, and after a passing `check_change`.

CONTROLS:

- `a_crlf_checkout_of_an_approved_delta_stays_intact`: uses
  `approved_delta_body_as_a_crlf_checkout_writes_it`. Expects `Some(true)` and the same lists.
- `restoring_or_reapproving_an_edited_delta_restores_intact`.
- `approved_delta_summaries_are_empty_outside_active_approved_states`: draft, a stale-scope
  approved change (`affected_paths` edited), accepted, archived and `no_spec_change` changes all
  return `[]`.
- `approved_delta_summaries_agree_with_changed_approved_deltas`: for every fixture above, assert
  both invariants in `design.md`.
- `delta_binding_refactor_keeps_existing_refusals`: `ensure_approved_delta_bodies_unchanged` and
  `changed_approved_delta_modules` give the same `Ok`, `Err` strings and lists as before. The
  existing #704, #719 and #730 tests and `status_reports_an_edited_approved_delta_as_an_invalid_approval`
  stay green unchanged.

## Integration test (`tests/integration/change.rs`)

`status_and_show_json_report_approved_delta_requirements` runs through the built binary under a
workflow-v2 `sdd.json`, following `status_json_and_text_report_an_edited_approved_delta`:

1. Create, answer, complete and approve a change with a fixture actor. Its delta adds
   `REQ-auth-002` and modifies one section.
2. Both `change status <id> --json` and `change show <id> --json` have `approved_deltas[0]` with
   `module == "auth"`, `delta_intact == true`, `requirements.added == ["REQ-auth-002"]` and
   `materialized == false`. The other top-level keys and `summary` keys equal the set on a80748f.
3. Re-encode the delta to CRLF. Still `true`.
4. Edit the delta body: `delta_intact == false`, no `requirements` key,
   `summary.changed_approved_deltas == ["auth"]`. Restore it.
5. Run `change check`. `materialized == true`.
6. Text `change status <id>` output is unchanged and contains no `REQ-auth-002`.
   `change list --json` has no `approved_deltas`.
7. A draft change's `change status --json` has `approved_deltas == []`.

## Re-run of the issue reproduction

Run the issue's `repro2.sh` against the built candidate. Section A should find
`REQ-shipping-002` in `status.json` and `show.json` under `approved_deltas`, and section B should
show `delta_intact: false`. Record the output in `context.md`.

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
| REQ-change-113 | Discriminators: `approved_delta_summaries_report_declared_requirements_and_sections`, `a_requirement_mentioned_in_prose_is_not_reported`, `an_edited_approved_delta_reports_no_approved_requirements`, `a_withdrawn_delta_binding_reports_every_module_not_intact`, `a_ledger_without_delta_digests_reports_unknown_integrity`, `materialized_turns_true_after_check_and_false_after_a_reapproved_correction`, `approved_delta_summaries_hold_in_implementing_and_verifying`. Controls: `a_crlf_checkout_of_an_approved_delta_stays_intact`, `restoring_or_reapproving_an_edited_delta_restores_intact`, `approved_delta_summaries_are_empty_outside_active_approved_states`, `approved_delta_summaries_agree_with_changed_approved_deltas`, `delta_binding_refactor_keeps_existing_refusals`. |
| REQ-cmd-change-023 | Integration: `status_and_show_json_report_approved_delta_requirements`. Existing REQ-cmd-change-005 coverage, including `status_json_and_text_report_an_edited_approved_delta`, keeps passing. |

No result is claimed here. Observed results are recorded after the tests are written and run.
