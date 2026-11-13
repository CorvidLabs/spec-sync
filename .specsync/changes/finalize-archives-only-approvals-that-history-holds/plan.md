---
change: finalize-archives-only-approvals-that-history-holds
artifact: plan
---

# Plan

## Approach

Before finalize writes anything, require the change's `approvals.json` to be the version at
`HEAD` and every committed version of it to extend the one before, apart from `relayed_by`.

## Out of scope

Legacy `change archive`; a finalize resumed after acceptance; archived packages' own history,
which the archive-introduction ledger checks already compare; two clones approving one change
(#532).

## Steps

1. Red tests in src/change_tests/committed_approvals.rs.
2. `ensure_approvals_ledger_held_by_history` and `approvals_ledger_rewrite`, called from
   `accept_change_with_gate` for the finalization gate, after the review check and before writes.
3. Workflow page paragraph.
4. Deltas once numbered; broad finalize, ship, archive and reopen tests; fmt and clippy.

## Risks

An honest flow that leaves `approvals.json` uncommitted at finalize would now be refused; every
writer appends, and `check --commit` commits it before review. The broad finalize, ship, archive
and reopen runs check the rest. One `git log` and one read per committed version are added to
finalize; they do not grow with other archives (REQ-change-121).

## Constraints consulted

REQ-change-121 (finalize cost), REQ-change-131 (refuse before writing), REQ-change-157
(verification commit check order), REQ-change-relay-repair-001 (relays are repaired, not gated).
