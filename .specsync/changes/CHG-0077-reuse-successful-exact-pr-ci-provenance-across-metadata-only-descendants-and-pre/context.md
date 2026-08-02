---
change: CHG-0077-reuse-successful-exact-pr-ci-provenance-across-metadata-only-descendants-and-pre
artifact: context
---

# Context

Trigger: PR #492 reproduced issues #488 and #489 while finalizing CHG-0075. A separately committed
review record had no hosted Trust check, so the archive child could not reuse the already-green
product tip. After review and archive were combined into one metadata child, archive integrity still
rejected the product ancestor because its latest trusted-policy result was the expected protected-file
failure. Earlier PR #486 and sandbox scenario 023 recorded the same orphan-parent and cancel-poison
classes.

Root cause: metadata-only workflows inspect only the immediate parent and trusted-policy lookup gives
newer cancelled or failed republications precedence over an earlier successful check for the same
exact SHA. Valid product evidence therefore becomes unreachable when one or more lifecycle-only
children are pushed.

Invariant: reusable evidence must come from the nearest successful first-parent product ancestor,
remain bound to the same pull request and exact SHA, be produced by the expected GitHub Actions
workflow/app, and pass every existing identity check. A later unsuccessful check may not invalidate an
earlier successful exact-SHA result, while missing, foreign, wrong-workflow, non-ancestor, stale, or
ambiguous evidence continues to fail closed.

Regression coverage: deterministic script fixtures reproduce an immediate parent without a check,
multiple metadata descendants, cancelled/failed checks newer than a successful check, foreign PR and
workflow identities, and an exhausted ancestor search. Hosted archive-only dogfood must reuse the
green product tip without rerunning the product matrix.

Port-review finding: the draft helper bounded ancestry but did not prove that every skipped child was
lifecycle metadata. That could have allowed an intervening code commit to borrow an older green tip.
The current child still uses the canonical lifecycle classifier; historical traversal accepts only
the exact same-change `review.json`/`review-attempts.json` pair and stops before every other edge.
Metadata-child check republications are skipped, and the first product boundary cannot borrow from
older product commits. Focused tests bind those invariants alongside second-parent, complete-lookup,
and hard-limit cases. Required CI also couples both helper paths to its own protected workflow, so a
later PR cannot weaken the PR-controlled script without triggering the base policy boundary.
