---
change: approvals-forked-across-clones-are-refused-until-one-fresh-approval
artifact: plan
---

# Plan

## Approach

Detect forked approvals ledgers in committed history during definition-approval validation, and
settle a fork with one fresh approval recorded after it.

## Out of scope

A network or cross-clone lock; detecting the race before either side commits; scoped reviews.

## Steps

1. Red unit tests in forked_approvals.rs.
2. `ensure_approval_history_not_forked` in `ensure_definition_approval_valid`.
3. Workflow page.
4. REQ delta once numbered; full unit suite, fmt and clippy.

## Risks

Every validation now reads the ledger's history: one `git log` per path, memoized per command,
and each version through the batch reader. The process-count tests in the full suite bound it.

## Constraints consulted

REQ-change-relay-repair-001 (relays repaired, not compared), the send-back history check
(CHANGE-12.c), and approve's accepted states.
