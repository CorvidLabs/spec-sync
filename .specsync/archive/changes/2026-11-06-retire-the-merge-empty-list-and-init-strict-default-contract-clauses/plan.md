---
change: retire-the-merge-empty-list-and-init-strict-default-contract-clauses
artifact: plan
---

# Plan

## Approach

Reword the merge and init clauses to the behavior the code and tests already have. Add contract pins that
fail on main. Add one CLI characterization test for the exit code that the reworded REQ-merge-001 now
states.

## Out of scope

Any runtime change, including `init --help` (decision e). #848's contracts, which have their own PR.
Historical change-log rows and archived packages.

## Steps

1. Failing first: `merge_contracts_call_a_failed_git_discovery_unknown` and
   `init_contracts_leave_sdd_off_for_git_repositories` fail on main (recorded).
2. Characterization: `default_merge_outside_git_exits_nonzero_and_names_the_unperformed_scan` passes on
   main and fails when the `Unknown` arm of `cmd_merge` stops exiting 1 (recorded).
3. After scope approval, edit the companions, add lesson planes and a CHANGELOG entry, then run
   `change check --commit`.
4. Run the verify lane on the pinned toolchain and the pre-push gate, then hand over for review.

## Risks

A sibling change could rewrite REQ-merge-001 or REQ-cmd-init-004 first. The stale-MODIFIED gate would then
refuse, and the fix is to rebase the delta and get it re-approved. The helper block in
tests/integration/helpers.rs is byte-identical to the one #848 adds, so the two PRs merge cleanly in either
order.

## Constraints consulted

change.3md: a successor that contradicts its predecessor does not retire it; someone has to. Leif's
decision (e) and the brief: no runtime change. Strictness stays: no gate, bound, timeout or existing
assertion is loosened.
