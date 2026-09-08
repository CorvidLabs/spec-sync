---
id: align-specsync-6-release-guidance-with-verified-lifecycle-and-validation-boundaries
state: implementing
type: documentation
base_commit: ffba9a32b664a7b3308350c162f0ac808f24efe3
---

# Align SpecSync 6 release guidance with verified lifecycle and validation boundaries

## Intent

Align SpecSync 6 release guidance with verified lifecycle and validation boundaries

## Affected Canonical Specs

- None

## Acceptance Criteria

- Public 6.0 guidance accurately separates structural checking from semantic proof and content-bound approval from authenticated identity; describes same-PR archival before merge and the actual scope of check versus change audit; warns against mixed 5.x/6.x lifecycle writers and explains explicit Trust binary pinning; labels legacy recovery commands; removes obsolete numeric-sequence and post-merge archival claims in the scoped pages; docs build and existing contract checks pass without changing executable behavior.

## No-spec Rationale

Correct public guidance to describe existing verified behavior and rollout constraints; no executable behavior, API, or canonical module contract changes.
