---
change: authenticate-finalized-policy-migrations-during-active-only-audit-without-weakening-policy-change-coverage
artifact: plan
---

# Plan

## Approach

Retain the current active-record policy check. If it has no owner, inspect only archive packages introduced by the actual delivery diff, using existing comparison-base and archive enumeration helpers. Authenticate relevant policy owners with the existing immutable archive validator and require the signed policy acceptance entry to equal the current exact entry. Report why relevant candidates fail rather than silently approving an unauthenticated scope claim.

## Out of scope

No altered approval identities, baseline edits, blanket archive acceptance, advisory ignores, port/runtime changes, merges, releases or deployment. Do not revalidate every historical archive during ordinary active audit.

## Steps

Implement a narrow internal coverage helper and regression fixtures; synchronize change contract/requirements/testing and record the lesson. Run the required native lane, strict canonical/scoped checks and hosted gates. Publish the upstream PR with exact candidate provenance and reproduce Algonest #48. Prepare Algonest consumption separately with an immutable candidate identity and digest; no unreviewed latest download or engine downgrade.

## Risks

Archive declarations alone could authorize unrelated or unsigned policy edits. Require actual same-delivery membership, authenticated finalized evidence and exact current policy binding. Partial clones must supply required authentic objects before a result is claimed.

## Constraints consulted

The change module lesson source requires bounded, selective archive authentication and keeps archive history separate from active checks. Existing strict policy and source ownership remain authoritative.
