---
change: an-in-flight-change-that-declares-a-shared-input-covers-an-accepted-change-for-now
artifact: plan
---

# Plan

## Approach

Let an in-flight later change that declares a changed input's module and path cover it for now,
report that as a warning, and let the accepted change archive with a record of the coverer.

## Out of scope

Downgrading every stale input; archived changes' evidence, which is authenticated history.

## Steps

1. Red tests.
2. Walk, validity, reason.
3. Audit, doctor, successor, retirement and reopen decisions.
4. coverage.json at archive; docs.
5. REQ delta once numbered; full suite.

## Risks

A new validity value reaches status JSON (`summary.terminal_evidence.validity`); consumers that
assert `exact` or `successor-covered` for an accepted change will see `provisionally-covered`.

## Constraints consulted

The accepted-input currency rules (successor coverage, REQ-change-135 introduction index), the
archive preflight, reopen's refusal of exact or successor-covered evidence, and retirement.
