---
change: leave-relayed-decisions-out-of-approval-succession-the-definition-digest-and-the-trusted-correction-history
artifact: plan
---

# Plan

## Approach

Apply the relay projection #900 already uses for the review and correction digests to the three comparisons that still see a relay, so CHANGE-13.h holds: a relay changes no gate, and an older binary that drops it can still reopen and finish the change.

## Out of scope

- Validating relays embedded in corrections, escaping names, recovery for correction-ledger relays, status reading review attempts, and relays nested in reopen events for repair-relays (G-relay-validate: #970, #972, #973, #974, #998, #1009).

## Steps

1. Failing tests first (done at fe9615c1), one per comparison, each also proving a non-relay edit is still caught.
2. Project relays out of `ledger_succession`.
3. Hash the correction ledger through `correction_digest_projection` in the definition digest.
4. Compare projected corrections in the trusted correction history.
5. Requirement, testing evidence, lesson and CHANGELOG.

## Risks

- These checks guard against tampering. The projection removes only `relayed_by`, and each test also checks that another edit is still refused.
- The definition digest of a change whose correction embeds a relay changes; only unreleased builds of main ever computed the old value.

## Constraints consulted

change.3md "Approvals are claims; relayed decisions (#633)"; "Keep it out"; REQ-change-046; the ledger-succession rule; never weaken a gate, bound or test.
