# Lesson bundle — run-the-effective-contract-gate-on-the-first-change-check-and-record-its-refusal-instead-of-leaving-a-stale-pass

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Run the effective contract gate on the first change check and record its refusal instead of leaving a stale pass current
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/sdd_version.rs, src/change/collect_requirement_ids.rs, src/change_tests/semantic_application_resolves_registry_backed_canonical_paths.rs, tests/integration/change.rs, site/src/content/docs/workflow.md, specs/change
- **Acceptance**: For an approved change whose declared module carries an unsuppressed validator warning (for example an undocumented export), the first `specsync change check <id>` after approval exits non-zero and names that warning. That is the verdict a second `change check`, `change check --commit`, `change finalize` and `change audit` already reach on the same tree. It is identical with `enforcement` set to `warn`, `enforce-new` or `strict`, because configured enforcement is not what refuses it.
- **Acceptance**: That refusal is recorded as an append-only failed attempt. `verification.json` reports `passed: false` and `verification-attempts.json` gains the attempt. `commands[]` keeps the scoped `specsync check --spec <name> …` entry and adds one entry with `success: false` whose command is `specsync change check <id> (effective contract: <module>, …)`, naming the gate and each refused declared module instead of claiming a `specsync check --strict` run.
- **Acceptance**: When a passed verification is already recorded and a later `change check` or `change check --commit` is refused by that gate for a declared module, the new failed attempt supersedes it. `change status` then names `specsync change check <id>` rather than `specsync change review`, and `--commit` commits nothing.
- **Acceptance**: A change whose declared modules carry no unsuppressed warning passes its first and later checks and records exactly the single scoped command it records today, without `--strict`. Explicit `--strict` behaves as today.
- **Acceptance**: Unchanged: `specsync check` and configured `enforcement` semantics; the gate's warning promotion and its ignore-rule and unauthored-stub suppressions; specs in scope only through `affected_paths`; `--no-spec-change` changes; gate failures not attributable to a module this change declares, which are still refused before any attempt is recorded; the acceptance and audit gates; accepted and archived evidence; `summary.strict_validation_required` and the built-in strict path and module classification.
- **Acceptance**: Regression tests fail on origin/main f240d37 and pass with the fix. They include discriminators for the first-check verdict under `warn` and under `strict`, for the recorded failure, and for the superseded pass; labelled controls for every unchanged case; and an integration test through the built binary covering `change check`, `change check --commit` and `change status --json`.

## Evidence

- Verification commit: `a982341d1f29e239b39044dfb3cb866f3834ad35`
- Base commit: `f240d379b9f023f5766e5c9b39508eb84e03e2bb`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Reported downstream as Chough BUG-0003. `change check` records `passed: true` for a tree that
every later lifecycle gate refuses. A failed `change check --commit` then leaves that pass
current, so `change status` keeps pointing at review.

## Reproduction (released SpecSync 6.1.1)

The reporter reproduced this in a disposable copy of Chough's `examples/shipping` with a
synthetic approval.

1. `.specsync/config.toml` has `enforcement = "strict"`. Approve a change for module `shipping`,
   then add an undocumented `pub fn` to a file `shipping` owns.
2. `specsync change check <id> --json` exits 0. `verification.json` records `passed: true` with
   commands `["specsync check --spec shipping"]`.
3. `specsync check --strict` fails on the undocumented export.
4. `specsync change check <id> --commit` exits 1 with "effective contract `shipping`: Undocumented
   export ...". It records no attempt: `verification-attempts.json` still holds one passed
   attempt.
5. `specsync change status <id>` still names `specsync change review <id> --reviewer <human>`, and
   `summary.strict_validation_required` is false.

## The report's premise does not hold: configured enforcement is not involved

`enforcement = "strict"` does not reject this tree.

Measured with installed 6.1.1 on a copy of `examples/shipping` plus an undocumented `pub fn`:
`specsync check` exits 0 under `strict`, `warn` and `enforce-new`. Only `specsync check --strict`
exits 1. That matches the code:

- `compute_exit_code` (`src/commands/mod.rs:1204`): `Strict` blocks on errors, and on warnings
  only when the `--strict` flag is passed.
- The `EnforcementMode` documentation (`src/types.rs:342`): "Warnings still pass unless `--strict`
  is given".
- Nothing under `src/change/` reads `enforcement`.

A throwaway unit test on origin/main f240d37 (scratch export, fixture approver, not committed)
ran with `enforcement = "warn"` and `enforcement = "strict"`. The results were identical:

| Step | Result under `warn` and `strict` alike |
|------|----------------------------------------|
| First `check_change` | `Ok(passed: true)`, 1 attempt, command `specsync check --spec auth`, state verifying, next action `change review`, `strict_validation_required` false |
| Second `check_change` | `Err("effective contract `auth`: 0/1 exports documented; effective contract `auth`: Undocumented export 'undocumented_probe' from src/auth.rs")`, still 1 attempt, latest still passed, next action still `change review` |
| `audit_project` | Reports the same two errors |
| Clean control | Both checks pass, 2 attempts |

## Cause

1. `validate_effective_contracts` (`src/change/collect_requirement_ids.rs:851`) promotes every
   unsuppressed validator warning to a hard error. This is documented at `:833` and narrowed by
   REQ-change-063's ignore-rule and unauthored-stub suppressions.
2. Its record filter (`:855`-`:861`) includes an active record only when
   `!record.canonical_applied || record.state == ChangeState::Verifying`. The filter arrived with
   audited reopening (e2184550, #353).
3. `change check` materializes before it verifies. `materialize_change_deltas` sets
   `canonical_applied = true` and `state = Implementing`
   (`src/change/sdd_version.rs:2948`-`:2949`). Only then does `verify_change_locked` run the gate
   (`:3040`). On the first check the gate therefore excludes the very change it is verifying, and
   with one active change it validates nothing. REQ-change-007 says "Effective-contract validation
   runs during verification and acceptance"; on the first verification it does not run for the
   change itself.
4. The scoped pass `evaluate_spec_code_sync` (`src/change/sdd_version.rs:3268`) fails on warnings
   only under explicit `--strict` (REQ-change-023). It records `passed: true` and moves the change
   to verifying (`:3075`-`:3076`).
5. Every later run sees `state == Verifying`, so the gate includes the change. It refuses at
   `:3040`, before `record_verification_attempt` (`:3075`), so nothing is recorded:
   - `verification.json` keeps the pass.
   - The project-input digest has not moved, so `verification_is_current` still holds and status
     points to review.
   - `change check --commit` fails in its first pass (`run_checked_commit`,
     `src/commands/change.rs:3983`) and commits nothing.
   - `finalize` refuses at acceptance (`src/change/historical_definition_digest_matches.rs:2226`).
   - `change audit` reports the error (`:4036`).

The branch name `0xleif/fix/change-check-honors-configured-strict` predates this measurement.
This change does not make `change check` honor configured enforcement. `research.md` explains why
that would be the wrong fix.

## The built-in strict classification

`change_requires_strict_validation` (`src/change/sdd_version.rs:3372`) hard-codes SpecSync's own
paths (`src/change.rs`, `src/mcp.rs`, `src/github.rs`, `.github/workflows/release.yml`,
`.github/workflows/post-merge-archive.yml`) and modules (`change`, `mcp`, `github`,
`authentication`, `security`). That is this repository's dogfooding baked into product code.

- Since REQ-change-091 it changes nothing about verification. It sets only the advisory
  `summary.strict_validation_required` (`src/change/historical_definition_digest_matches.rs:3466`),
  plus `verification_commands_for_change`, which is dead outside tests (`sdd_version.rs:3315`).
- `src/change.rs` has been a 5-line `include!` shim since #810, so that entry no longer matches
  the implementation under `src/change/`.
- This repository's own `.specsync/sdd.json` already lists the same files in `strict_paths`.
- It played no part in this defect: `shipping` is not classified, and the gate refuses
  regardless.

It is left unchanged here. Removing it, and correcting the docs sentence that says the
classification "adds validators" (`site/src/content/docs/workflow.md:84`,
`site/src/content/docs/cli.md:421`), is a separate change for the human to decide.

## Human intent

This change links to existing criteria and adds none:

- SHIP-1.a: the code is checked against the contract as it now stands.
- SHIP-1.d: the evidence matches the tree it describes.
- SHIP-3.a: finalizing needs verification that is still current.
- CHANGE-7: the single next thing to do.

CHECK-3 and CHECK-3.a (an undocumented export is a warning unless promoted) are unchanged for
`specsync check`.

## From the change's design.md

# Design

## Part A: the gate sees the change it is verifying

`verify_change_locked` passes the effective-contract gate a roster in which the change under
verification is treated as verifying. That is the state this attempt records. The gate then
validates that change's declared modules on every check, the first one included, by the same
rule finalization and audit apply. The filter in `validate_effective_contracts` stays as it is for
every other caller.

## Part B: a refusal of a declared module is recorded, not just returned

`EffectiveContractOutcome` gains private per-module attribution, so verification can tell two
kinds of refusal apart:

- A refusal of a module this change declares is a spec↔code verdict about this change. It is
  folded into the attempt.
- Every other gate failure stays a pre-attempt refusal, exactly as today. That covers ordering,
  workspace creation, an unresolvable canonical spec, or a module only another active change
  declares.

Order inside `verify_change_locked`:

1. Approval, definition, delta, dependency and conflict checks. Unchanged.
2. Effective-contract gate. Non-attributable errors return at once, as today. Errors for declared
   modules are held.
3. Tasks and requirement-evidence completeness. Unchanged pre-attempt refusals.
4. The scoped pass. Unchanged rule and unchanged command string.
5. One attempt is recorded:
   - `commands[0]` is the scoped entry.
   - When the gate held errors, `commands[1]` is
     `CommandEvidence { command: "specsync change check <id> (effective contract: <m>, …)", success: false, exit_code: Some(1) }`,
     with modules sorted.
   - `passed` stays the conjunction of the entries.
6. State moves to verifying, as for any recorded attempt. On failure the error is today's
   `verification failed: …; see commands[] in …/verification.json`, with the gate's findings in
   the summary.

## Resulting behavior

| Situation | origin/main f240d37 | After |
|-----------|---------------------|-------|
| First check, declared module has an undocumented export | `passed: true`, exit 0, next `change review` | `passed: false`, exit 1 naming the export, next `change check` |
| Later check or `--commit`, same tree | exit 1, nothing recorded, pass stays current | exit 1, failed attempt recorded, pass superseded |
| Clean declared modules | one scoped command, pass | identical |
| `--strict` | scoped command with `--strict` | identical; the gate entry is added only when the gate refuses |
| Warning only in a path-scoped spec, or a `--no-spec-change` change | not gated by the effective contract | identical |
| Gate failure caused only by another change's module | pre-attempt refusal | identical |
| `enforcement` = warn, enforce-new or strict | irrelevant to the gate | still irrelevant |

## Contract edits (see `deltas/change.md`)

- ADDED REQ-change-112: the gate runs on every check, and its refusal is recorded.
- MODIFIED REQ-change-023: the evidence bullet names the gate entry.
- MODIFIED SPEC SECTION Contract, item 5: the same, so the spec does not keep a sentence that the
  new evidence contradicts.

## Compatibility

- Persisted shape is unchanged, because `commands` is already a list. A failed attempt may carry
  two entries.
- Readers that assume one entry, or that a pass is the only first-check outcome, see a new case.
- Recorded passes from 6.1.1 stay readable and stay current until the next check. If the gate
  refuses that check, it records a failure that supersedes them.
- No digest, schema version or archive changes.
- `summary.strict_validation_required` and `change_requires_strict_validation` are untouched.

## From the change's testing.md

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

## Observed results (2026-10-31)

Labels were confirmed against an export of f240d37 with only the new tests applied (no fix):

- Failed on f240d37, as labelled DISCRIMINATOR: `first_change_check_refuses_an_undocumented_export_in_a_declared_module` and `first_check_and_later_gates_agree_on_a_declared_module_warning` (the first `check_change` returned a pass), `a_gate_refusal_appends_a_failed_attempt_after_a_recorded_pass` (no attempt was recorded), and the integration test `change_check_records_the_effective_contract_refusal_on_the_first_run` (the first check succeeded).
- Passed on f240d37, as labelled CONTROL: `a_clean_declared_module_still_records_one_scoped_command`, `explicit_strict_verification_is_unchanged`, `a_warning_only_in_a_path_scoped_spec_keeps_the_scoped_rule`, `a_no_spec_change_delivery_is_not_gated_by_the_effective_contract`, `a_gate_failure_in_another_changes_module_is_still_refused_before_recording`, and the existing `status_does_not_advertise_strict_unless_strict_was_requested`, `change_check_fails_when_specs_and_code_drift` and `change_check_ignores_drift_in_an_unowned_spec`.
- `gate_error_attribution_drops_no_error` exercises the new private attribution, so it cannot compile on f240d37 and was left out of that run.
- With the fix, all of them pass. The full `cargo test` run passes: 2,609 unit tests and 463 integration tests.

Deviations:

- `first_check_and_later_gates_agree_on_a_declared_module_warning` asserts only that acceptance refuses. With the fix the latest attempt has failed, so `accept_change` refuses earlier, on stale verification, and does not reach the gate's message.
- The unit fixtures are not Git repositories. The integration test is.
- The path-scoped control declares `src/` as its affected path, because approval refuses an exact path owned by an undeclared module.

## Where these lessons go

- `specs/change/change.3md`
