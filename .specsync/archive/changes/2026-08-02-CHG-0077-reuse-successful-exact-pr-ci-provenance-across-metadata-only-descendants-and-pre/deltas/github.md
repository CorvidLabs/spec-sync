## MODIFIED

### REQUIREMENT REQ-github-008

Review-only and archive-only descendants SHALL reuse successful CI provenance from the nearest
eligible first-parent product ancestor without allowing later unsuccessful republication or unrelated
GitHub evidence to authorize the change.

Acceptance Criteria

- Reuse walks at most 32 first parents. The current child must classify as exact review/archive
  metadata, and traversal stops before any earlier child that is neither exactly one same-change
  `review.json` plus `review-attempts.json` update nor a matching workflow-v2 archive move whose
  finalization binds the exact parent commit and tree.
- Metadata-child check republications are not treated as fresh product evidence, and a product
  boundary with no eligible success cannot borrow an older product commit's checks.
- The provenance helper and its focused test cannot change without also changing the separately
  protected required-CI workflow.
- Implementation-ready, scoped-review, and Trust evidence share one product ancestor; the two CI
  checks share one workflow run.
- Every reusable job check is successful, exact-SHA-bound, GitHub-Actions-authored, bound to the same
  pull request and repository, produced by the expected workflow, and linked to its exact successful
  workflow job and selected check identity.
- A newer cancelled or failed trusted-policy publication does not override an earlier authenticated
  success for the same exact SHA, including a failed rerun of the same workflow run; the successful
  publication remains bound to its immutable run attempt even when GitHub rewrites its display URL.
- Missing, foreign, stale, wrong-workflow, second-parent, malformed, unsuccessful-only, over-limit,
  or ambiguous evidence fails closed.
- Eligible metadata descendants do not rerun the full product matrix.
