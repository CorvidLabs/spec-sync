## MODIFIED

### REQUIREMENT REQ-github-008

Review-only and archive-only descendants SHALL reuse successful CI provenance from the nearest
eligible first-parent product ancestor without allowing later unsuccessful republication or unrelated
GitHub evidence to authorize the change.

Acceptance Criteria

- Reuse walks at most 32 first parents. The current child must classify as exact review/archive
  metadata, and traversal stops before any earlier child that is not exactly the same change's
  `review.json` plus `review-attempts.json` update.
- Metadata-child check republications are not treated as fresh product evidence, and a product
  boundary with no eligible success cannot borrow an older product commit's checks.
- The provenance helper and its focused test cannot change without also changing the separately
  protected required-CI workflow.
- Implementation-ready, scoped-review, and Trust evidence share one product ancestor; the two CI
  checks share one workflow run.
- Every reusable check is successful, exact-SHA-bound, GitHub-Actions-authored, bound to the same
  pull request and repository, and produced by the expected workflow.
- A newer cancelled or failed trusted-policy publication does not override an earlier authenticated
  success for the same exact SHA.
- Missing, foreign, stale, wrong-workflow, second-parent, malformed, unsuccessful-only, over-limit,
  or ambiguous evidence fails closed.
- Eligible metadata descendants do not rerun the full product matrix.
