---
id: CHG-0046-make-lifecycle-verification-workflows-evaluate-the-exact-pull-request-head-while
state: implementing
type: operations
base_commit: 03aa191badb0049986d39aaebc7dec4855ce850f
---

# Make lifecycle verification workflows evaluate the exact pull-request head while preserving synthetic-merge validation in ordinary build and test lanes

## Intent

Make lifecycle verification workflows evaluate the exact pull-request head while preserving synthetic-merge validation in ordinary build and test lanes

## Affected Canonical Specs

- None

## Acceptance Criteria

- On pull_request events the ci.yml spec-check job and trust.yml trust job checkout github.event.pull_request.head.sha with fetch-depth 0; on push events both fall back to github.sha; all other CI checkout steps remain unchanged and continue testing the synthetic merge; workflow YAML parses; strict SpecSync and candidate Trust verification pass.

## No-spec Rationale

This corrects GitHub Actions checkout semantics for lifecycle verification lanes only; it does not change SpecSync runtime behavior, canonical requirements, or the public contract.
