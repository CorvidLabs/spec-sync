---
change: CHG-0077-reuse-successful-exact-pr-ci-provenance-across-metadata-only-descendants-and-pre
artifact: requirements
---

# Requirements

## REQ-github-008: Metadata-descendant CI provenance reuse

Review-only and archive-only descendants SHALL reuse successful CI provenance from the nearest
eligible first-parent product ancestor without allowing later unsuccessful republication or unrelated
GitHub evidence to authorize the change.

Acceptance Criteria

- Reuse walks only the bounded first-parent ancestry of the current pull request.
- A reusable check is successful, exact-SHA-bound, GitHub-Actions-authored, PR-bound, and produced by
  the expected workflow.
- Newer cancelled or failed checks or workflow reruns do not override an earlier successful
  exact-SHA trusted-policy result; the successful publication binds its immutable run attempt.
- Missing, foreign, stale, wrong-workflow, non-ancestor, malformed, or ambiguous evidence fails
  closed.
- Metadata descendants do not rerun the full product matrix when eligible ancestor evidence exists.
