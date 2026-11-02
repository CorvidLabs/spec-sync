---
change: acknowledge-anchored-historical-receipts-explicitly-without-weakening-normal-transition-proof
artifact: plan
---

# Plan

## Approach

Add an explicit audited scope request to existing definition-bound answers, versioned historical closing evidence, and a separate validation branch that authenticates anchored old receipts. Retain strict normal succession byte-for-byte.

## Out of scope

No new release or merge without applicable gates; no rewritten signed histories, widened capture bounds, ignored paths, general corruption bypass, unsupported symlink/gitlink migration, or claims that old product transitions were proven.

## Steps

Record approved intent; implement the opt-in command and evidence schema; authenticate exact anchored packages and old manifests; require fresh scope approval, verification and review; add adversarial tests and explicit output disclosure; qualify the source; use the candidate to close the consumer receipts and rerun site CI.

## Risks

Historical acknowledgement must never masquerade as normal transition proof. A separately versioned evidence schema, definition-bound opt-in, explicit unproven status and strict negative controls enforce that boundary. Ordinary file receipt paths only; protected tool metadata remains excluded.

## Constraints consulted

RECOVER-2, explicit user approval, existing strict succession and immutable receipt rules, repository mandatory pre-push/full verification and Trust gates.
