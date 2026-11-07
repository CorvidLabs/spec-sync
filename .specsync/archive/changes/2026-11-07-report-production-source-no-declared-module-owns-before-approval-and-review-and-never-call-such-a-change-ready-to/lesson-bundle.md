# Lesson bundle: report-production-source-no-declared-module-owns-before-approval-and-review-and-never-call-such-a-change-ready-to

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Report production source no declared module owns before approval and review, and never call such a change ready to finalize
- **Kind**: BugFix
- **Specs**: change, cmd_change
- **Paths**: CHANGELOG.md, src/change/, src/commands/change.rs, src/change_tests/, tests/integration/change.rs
- **Acceptance**: change approve refuses a no-spec-change draft whose existing affected production-source path a module owns, naming the path, the owning spec and module, and specsync change answer <id> affected_specs <modules>; running that command lets the draft approve.
change ship-status and change ship --dry-run run the canonical-ownership check finalize runs over the delivery inputs: a production-source input no declared module owns is a blocker naming the path, the owning module, and the recovery (specsync change send-back <id>, then specsync change answer <id> affected_specs <modules>); neither says ready to finalize, and the dry run exits non-zero.
The integration test with a directory-scoped no-spec-change change runs that recovery and ships; it and the approve unit test fail on main 9dd2affd and pass with the fix.
Ownership is decided exactly as before (REQ-change-053): a path that does not exist yet is still enforced at finalize, an owned change ships unchanged, and no gate, bound or timeout is relaxed.

## Evidence

- Verification commit: `03dd0630df811830aa028a8113bd91b1b43c250b`
- Base commit: `9dd2affdca4628156835a6a0488e32a2f4702e45`
- Verified by: `specsync check --spec change --spec cmd_change`

## From the change's context.md

# Context

Issues #986 (found shipping #980) and #956 (found shipping #900's covering change), both the same gap. A `--no-spec-change` change with no declared specs whose scope delivers production source that a module owns passes `change approve`, `check --commit` and `review`. Then `ship-status` and `ship --dry-run` say "ready to finalize", and only `change ship` refuses with "acceptance input `<path>` is production source without deterministic canonical ownership". The refusal names no recovery. The recovery is a send-back, declaring the owning module, re-approval, re-verification and a fresh review, so one human approve-and-review cycle is wasted every time.

Root cause:
- `validate_declared_path_ownership` returns early for no-spec-change changes with no declared specs, as REQ-change-053 says ("finalize still enforces").
- Approve cannot see which owned files a directory scope (for example `src/billing/`) will deliver.
- Ship preflight and ship-status never run finalize's ownership resolution (`acceptance_input_owners`).

Intent: SHIP-4.a ("The answer names the stage I am on and the one thing holding it") and CHANGE-10 ("When `change approve` refuses an affected path that no module owns, the error says which spec to edit").

Out of scope: who owns what (REQ-change-053's resolution rules are unchanged), owner corrections for reopened changes, and paths that do not exist yet at approve, which are still enforced at finalize and now also at ship preflight.

## From the change's design.md

# Design

Two checks, both reusing finalize's own ownership resolution, so the answer can never disagree with finalize.

1. **Approve** (`validate_declared_path_ownership`): stop skipping a no-spec-change change with no declared specs. Each existing affected production-source path is resolved the same way, so with no declared module it is unowned and refused. The refusal keeps today's per-path hints (REQ-change-120). It gains one line for a draft: `specsync change answer <id> affected_specs <modules>`, where the modules are the declared ones plus every undeclared module that owns a refused path.
2. **Ship preflight and ship-status:** a read-only ownership preflight walks the delivery inputs finalize would sign (the same discovery and `record_covers_project_path` filter as the acceptance manifest) and calls `acceptance_input_owners` with `UnownedProductionSource::Reject`.
   - Every input that fails with the ownership error becomes one blocker.
   - The blocker names the path, the owning module (by the same hint as approve), and the recovery: `specsync change send-back <id>`, then `specsync change answer <id> affected_specs <modules>`.
   - `ready_to_finalize` is false, ship-status lists the blocker, and `ship` (including `--dry-run`) exits non-zero without mutating anything, as for open tasks (REQ-cmd-change-031).

Rejected:
- Expanding directory scopes at approve: a directory covers files the change may never touch, so it would refuse changes finalize accepts.
- Keeping the check only at finalize: that is the bug.

## From the change's testing.md

# Testing

1. Unit `approve_refuses_a_no_spec_change_draft_over_owned_production_source_and_names_the_answer`: a no-spec-change draft with no specs over an existing `src/billing/charge.ts` that `billing` lists.
   - Approve refuses, naming the path, `specs/billing/billing.spec.md`, module `billing` and `specsync change answer <id> affected_specs billing`.
   - Running that answer, then approve, succeeds.
   - Fails on main 9dd2affd (approve succeeds).
2. Integration `ship_readiness_names_unowned_production_source_before_finalize_and_the_recovery_ships`: a directory-scoped no-spec-change change, approved, checked and reviewed.
   - `ship-status` must not say ready and names the path, `billing`, `specsync change send-back <id>` and the answer.
   - `ship --dry-run` exits non-zero naming the answer.
   - Then send-back, answer, approve, check, review and ship archive it.
   - Fails on main 9dd2affd (ship-status said "ready to finalize"; the real ship refuses with the #956 error, reproduced).
3. The existing approve-ownership, open-task blocker and ship tests stay green.
4. `specsync change check <id> --commit`, `specsync change audit --strict`, `fledge lanes run verify` on pinned 1.89, pre-push.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| `REQ-change-053` | `approve_refuses_a_no_spec_change_draft_over_owned_production_source_and_names_the_answer`: a justified no-spec-change draft with no declared specs is ownership-checked at approve, and its existing owned production path is refused. |
| `REQ-change-120` | `approve_does_not_recommend_a_module_whose_listing_spec_is_not_canonical`: a non-canonical listing spec is explained, not recommended (Codex P2 on #999). The approve test above: the refusal names `specs/billing/billing.spec.md`, module `billing` and, for the draft, `specsync change answer <id> affected_specs billing`; running it clears the refusal. The existing `approve_ownership_refusal_*` tests keep the wording for declared changes. |
| `REQ-cmd-change-008` | `ship_readiness_names_unowned_production_source_before_finalize_and_the_recovery_ships`: ship-status names the blocker and the recovery, `ship --dry-run` exits non-zero, and the named recovery ships. `the_ownership_recovery_names_a_repair_for_every_unowned_input` and `an_archive_candidate_with_unowned_production_source_is_not_told_to_ship` cover the recovery for paths no module claims and the `change status`/`change pr` archive guidance (Codex P2s on #999). |

## Results

- Both new tests failed on main 9dd2affd: approve accepted the draft, and ship-status said "ready to finalize" while `ship` refused with the #956 error. Both pass with the fix; the integration test runs the named recovery and ships.
- The ship, approve, ownership, open-task, scoped-review and finalize suites are green: 91 unit and 23 integration tests.
- The three Codex P2s on #999 were fixed test-first (committed red at e0167d71). The integration fixtures that approved no-spec-change changes over `src/lib.rs` now declare `--spec lib`, with their assertions unchanged. 130 unit and 33 integration ship/approve/status/finalize tests are green.

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
