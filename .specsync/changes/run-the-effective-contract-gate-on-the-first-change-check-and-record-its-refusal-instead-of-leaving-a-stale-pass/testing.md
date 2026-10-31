---
change: run-the-effective-contract-gate-on-the-first-change-check-and-record-its-refusal-instead-of-leaving-a-stale-pass
artifact: testing
---

# Testing

Each new test carries an honest label. A DISCRIMINATOR fails on origin/main f240d37 and passes
with the fix. A CONTROL passes on both. Labels are confirmed against a build of unfixed f240d37
from a separate checkout.

## Unit tests

These go in `src/change_tests/semantic_application_resolves_registry_backed_canonical_paths.rs`,
next to `change_check_fails_when_specs_and_code_drift` and
`status_does_not_advertise_strict_unless_strict_was_requested`.

The fixture is `completed_section_only_current_record` in a Git repository, with `src/auth.rs`
exporting an undocumented `pub fn` and approved by a fixture actor. This is the fixture the
measurement in `context.md` used.

DISCRIMINATORS:

- `first_change_check_refuses_an_undocumented_export_in_a_declared_module`: runs once with
  `enforcement = "warn"` and once with `enforcement = "strict"` in `.specsync/config.toml`. The
  first `check_change` returns `Err` naming `Undocumented export 'undocumented_probe'`.
  `verification.json` has `passed: false` with two entries:
  - `commands[0].command == "specsync check --spec auth"` with success true,
  - `commands[1].command == "specsync change check <id> (effective contract: auth)"` with success
    false.
- `a_gate_refusal_appends_a_failed_attempt_after_a_recorded_pass`: records a pass with a clean
  tree, then writes 6.1.1's shape: the undocumented export under an unchanged project digest,
  with the recorded pass restored. Asserts that the next `check_change` appends a failed attempt,
  that `verification-attempts.json` holds both, and that `summarize_change.next_action` names
  `change check`, not `change review`.
- `first_check_and_later_gates_agree_on_a_declared_module_warning`: for the same tree,
  first-check failure, second-check failure, `audit_project` errors, and `accept_change` refusal
  all name the same warning.

CONTROLS:

- `a_clean_declared_module_still_records_one_scoped_command`: first and second checks pass with
  `commands == ["specsync check --spec auth"]` and no `--strict`.
- `explicit_strict_verification_is_unchanged`: `check_change_with_strict(.., true)` records the
  `--strict` scoped command as today.
- `a_warning_only_in_a_path_scoped_spec_keeps_the_scoped_rule`: the warning sits in a spec reached
  only through `affected_paths`. The non-strict pass still passes.
- `a_no_spec_change_delivery_is_not_gated_by_the_effective_contract`.
- `a_gate_failure_in_another_changes_module_is_still_refused_before_recording`: no new attempt is
  appended.
- `gate_error_attribution_drops_no_error`: the union of attributed and returned errors equals
  `validate_effective_contracts(..).errors`.
- The existing `status_does_not_advertise_strict_unless_strict_was_requested`,
  `change_check_fails_when_specs_and_code_drift`, `change_check_ignores_drift_in_an_unowned_spec`
  and the REQ-change-063 suppression tests (`effective_contract_exempts_stub_sections_no_active_change_authored`
  and others) stay green unchanged.

## Integration test

This goes in `tests/integration/change.rs`, through the built binary.

`change_check_records_the_effective_contract_refusal_on_the_first_run`:

1. In a temp Git repository with `enforcement = "strict"`, approve a change with a fixture actor
   and add an undocumented export.
2. Assert:
   - `change check <id> --json` exits 1,
   - `verification.json` has `passed: false` and the gate entry,
   - `change check <id> --commit` exits 1 and creates no commit (`git rev-parse HEAD` is
     unchanged),
   - `change status <id> --json` names `specsync change check <id>`.
3. Document the export in the spec's Public API.
4. Assert that `change check` passes and records one scoped command.

## Regression lanes

- `cargo test change::`.
- `cargo test commands::change::`.
- `cargo test --test integration change::`.
- `fledge lanes run verify`.
- `specsync check --strict --require-coverage 100`.
- `specsync change audit`.
- `fledge lanes run pre-push` before every push.
- `fledge trust verify` before calling the work complete.

## Requirement evidence

| Requirement | Required evidence |
|-------------|-------------------|
| REQ-change-112 | Discriminators: `first_change_check_refuses_an_undocumented_export_in_a_declared_module` (under `warn` and `strict`), `a_gate_refusal_appends_a_failed_attempt_after_a_recorded_pass`, `first_check_and_later_gates_agree_on_a_declared_module_warning`. Controls: clean module, explicit `--strict`, path-only scope, `--no-spec-change`, another change's module, attribution completeness. Integration: `change_check_records_the_effective_contract_refusal_on_the_first_run`. |
| REQ-change-023 | The gate-entry assertions above. `status_does_not_advertise_strict_unless_strict_was_requested`, `an_empty_scope_is_named_rather_than_written_as_a_project_wide_pass`, `mixed_scope_evidence_names_the_missing_module_but_does_not_rerun_faithfully` and `the_spec_filter_name_is_the_stem_filter_specs_matches_not_the_declared_module` stay green unchanged. |

No result is claimed here. Observed results are recorded after the tests are written and run.
