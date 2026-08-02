---
change: CHG-0077-reuse-successful-exact-pr-ci-provenance-across-metadata-only-descendants-and-pre
artifact: research
---

# Research

## Reproduced incidents

- PR #486 / sandbox scenario 023: product, review, and archive tips were pushed close together;
  immediate-parent reuse found no check and in-flight trusted-policy publication was cancelled.
- PR #492: the product tip `76aa53e` had green product and Trust checks, but a separate review-only
  parent had none. Combining review/finalization restored immediate-parent Trust reuse, while archive
  integrity still rejected the product tip because its newest trusted-policy check was the expected
  protected-workflow failure.

## Existing implementation candidate

Draft PR #490 contains a bounded helper and fixtures in commit `807f4619`. The reusable slice is
limited to ancestor check lookup, trusted-policy result selection, Trust/archive workflow wiring, and
their tests. The draft's `ship-status`, pre-push, lifecycle packages, and duplicate CHG-0074 CI work
are unrelated and must not be carried forward.

## Security constraints

Ancestor walking cannot weaken identity. Each candidate remains exact-SHA-bound, belongs to this PR
and repository, comes from the official GitHub Actions app and expected workflow path, and is
successful. Search is first-parent-only and bounded by committed lifecycle limits. Success-preferred
selection applies only among checks for the same exact SHA and expected identity; it never converts
an unsuccessful-only set into success.

Reviewing the draft slice found that first-parent traversal alone was insufficient: without checking
each edge, an unverified code child could be skipped on the way to older green evidence. The repair
therefore keeps the existing classifier on the current child, admits only exact historical scoped-
review pairs while walking, and requires implementation-ready, scoped-review, and Trust checks to
converge on one product ancestor; the two CI checks must also resolve to one workflow run.
The first non-review commit is a terminal product boundary: its own exact checks either authorize
reuse or fail, and checks republished on metadata children or older product commits cannot substitute.

## PR #494 review findings

- Separate workflow-v2 archive commits are lifecycle metadata, but the original walker treated the
  first prior archive as a product boundary. Historical archive traversal now requires the archived
  state plus finalization evidence bound to the exact parent commit and tree.
- A generic Actions check can present a run-level URL without proving that the named required job
  succeeded. Reuse now requires an exact job URL and authenticates the job's run, SHA, name,
  conclusion, and selected check-run identity.
- GitHub may rewrite a custom policy check's requested workflow URL to a canonical check URL
  (CHG-0076). Therefore the cancel-poison repair filters canonical-URL candidates to successful
  matching policy runs rather than requiring one publication of any conclusion; an explicit workflow
  URL still selects only its named run.
