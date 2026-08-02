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
- Historical archive traversal cannot trust labels alone. The final matcher derives the complete
  affected path/spec set, validates canonical owners, authenticates payload bytes with one bounded
  Git object reader, reconstructs historical sequence evidence, and reproduces every signed digest.
  Review attempts added during finalization must bind the exact parent and remain independent from
  the scope approver.
- Successful policy publications are tried newest-first, but the fallback is bounded to eight
  candidates and 30 seconds total so malformed republications cannot create unbounded process/API
  amplification.
- Recursive `git ls-tree` output omits directory objects even though Rust legitimately records a
  covered directory without a trailing slash as `non_file`. The matcher now requests tree objects
  in the same bounded inventory, excludes them from ordinary file discovery, expands the directory
  scope to every tracked descendant, and authenticates the explicit non-file entry as a tree with
  the signed zero mode and empty payload. Existing tree objects cannot be represented as `missing`.
- Sequence-ledger reconstruction used a hard-coded 256-commit ceiling while Rust accepts the
  committed `scoped_review_max_descendants` limit (currently 1000). The helper now reads that
  canonical value, validates it as a non-boolean integer in `1..=1000`, requests one overflow
  sentinel, and fails closed on invalid configuration or excess history.
