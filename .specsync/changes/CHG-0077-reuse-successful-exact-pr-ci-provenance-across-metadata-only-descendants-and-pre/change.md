---
id: CHG-0077-reuse-successful-exact-pr-ci-provenance-across-metadata-only-descendants-and-pre
state: accepted
type: feature
base_commit: 7b6b10d1f54fd1fc32e9bbe936cf23ab39958e0b
---

# Reuse successful exact-PR CI provenance across metadata-only descendants and prevent later cancelled or failed republishing from poisoning an earlier successful exact-SHA result

## Intent

Reuse successful exact-PR CI provenance across metadata-only descendants and prevent later cancelled or failed republishing from poisoning an earlier successful exact-SHA result

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- A review-only or archive-only metadata descendant reuses the nearest successful first-parent product CI evidence only when the check is successful, GitHub-Actions-authored, bound to the same pull request and exact ancestor SHA, and produced by the expected workflow; later cancelled or failed checks or workflow reruns cannot override an earlier successful exact-SHA trusted-policy result whose immutable run attempt is authenticated; missing, foreign, stale, wrong-workflow, non-ancestor, or ambiguous evidence fails closed; focused tests reproduce PR #492's orphan-parent and cancel-poison failures and pass after the repair.

## No-spec Rationale

Not applicable
