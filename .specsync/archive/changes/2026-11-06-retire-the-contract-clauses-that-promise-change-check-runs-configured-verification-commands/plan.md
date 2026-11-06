---
change: retire-the-contract-clauses-that-promise-change-check-runs-configured-verification-commands
artifact: plan
---

# Plan

## Approach

Retire or reword every current clause that says or implies that SpecSync executes configured
verification commands, so each defers to REQ-change-023. Add contract-pin tests that fail on main and
pass afterwards. Add characterization tests for the two reworded criteria that had no dedicated test.

## Out of scope

Any runtime change: no warning (#893), no execution, no deletion of dead helpers, and no change to
`init --help`. Merge and init contracts belong to #884 (its own change). Historical change-log rows,
archived packages and earlier lesson planes are not touched.

## Steps

1. Failing first: `change_contracts_give_one_answer_on_configured_verification_commands`,
   `comparison_pages_do_not_credit_specsync_with_running_configured_tests` and
   `comment_contracts_say_no_configured_verification_command_runs` fail on main (recorded).
2. Characterization: `missing_requirement_evidence_is_distinct_from_missing_semantic_evidence` and
   `editing_only_the_verification_command_list_keeps_the_bootstrap_exemption` pass on main and fail
   under a mutation of the code they guard (recorded).
3. After scope approval, edit the companions and site rows, add lesson planes and a CHANGELOG entry,
   then `change check --commit` to materialize the deltas.
4. Run the full local verify lane and pre-push, then hand over for review.

## Risks

A sibling PR could rewrite one of the modified requirement blocks first. The stale-MODIFIED gate
(REQ-change-122) would then refuse, and the fix is to rebase the delta and get it re-approved.
Contract pins match exact wording, so a future rewording needs a deliberate test edit. That is the
point (SHIP-1.c.2).

## Constraints consulted

change.3md: "A successor that contradicts its predecessor does not retire it; someone has to." This
change retires the predecessors. Leif's decisions d and e: no runtime change. Strictness stays: no gate,
bound, timeout or existing assertion is loosened.
