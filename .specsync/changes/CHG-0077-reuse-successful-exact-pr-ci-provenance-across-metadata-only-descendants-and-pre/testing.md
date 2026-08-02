---
change: CHG-0077-reuse-successful-exact-pr-ci-provenance-across-metadata-only-descendants-and-pre
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-github-008` | Ancestor-reuse and trusted-policy fixtures plus hosted archive-only sandbox dogfood. |

## Characterization

- Immediate-parent lookup fails when a green product tip is followed by an unpushed review child.
- Latest-result selection rejects an exact SHA when a later cancellation follows a successful check.

## Focused regressions

- Select the nearest successful check on bounded first-parent ancestry.
- Reject second-parent, non-ancestor, another-PR, another-repository, wrong-workflow, wrong-App,
  malformed, unsuccessful-only, and over-limit evidence.
- Prefer a successful trusted-policy check for the exact SHA over newer cancelled or failed checks.
- Preserve failure when no authenticated success exists.
- Confirm review/archive metadata-only classification and product-matrix skipping remain exact.
- Traverse a prior workflow-v2 archive only when its state and finalization bind the exact parent
  commit/tree; reject a mismatched binding.
- Reject reusable job checks with run-only URLs, wrong job names, or another check-run identity.
- Preserve canonical rewritten trusted-policy URLs while ignoring later failed/cancelled runs and
  rejecting multiple successful matching runs as ambiguous.

## Completion

- Run the two focused Python suites and lifecycle-workflow assertions while iterating.
- Run one final repository verification after review.
- In `CorvidLabs/spec-sync-sandbox`, push product plus metadata descendants without waiting and
  require Trust/archive reuse to bind the green product ancestor.
