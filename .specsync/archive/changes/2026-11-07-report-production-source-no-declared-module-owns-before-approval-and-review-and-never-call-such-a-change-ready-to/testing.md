---
change: report-production-source-no-declared-module-owns-before-approval-and-review-and-never-call-such-a-change-ready-to
artifact: testing
---

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
