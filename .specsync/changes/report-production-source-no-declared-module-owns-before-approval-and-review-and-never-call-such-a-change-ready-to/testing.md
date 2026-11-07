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

## Results

- Both new tests failed on main 9dd2affd: approve accepted the draft, and ship-status said "ready to finalize" while `ship` refused with the #956 error. Both pass with the fix; the integration test runs the named recovery and ships.
- The ship, approve, ownership, open-task, scoped-review and finalize suites are green: 91 unit and 23 integration tests.
