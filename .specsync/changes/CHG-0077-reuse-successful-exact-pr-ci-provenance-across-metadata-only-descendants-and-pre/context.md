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

PR #494 review trigger: three P1 findings arrived after the first finalization. The implementation
recognized only review-pair ancestors, accepted generic run-level check URLs, and selected policy
runs from all publications instead of excluding later unsuccessful republications. Root cause:
historical metadata and check/run identity were modeled too coarsely. Durable invariant: prior
workflow-v2 archive edges require an exact parent commit/tree finalization binding; generic reusable
checks must authenticate their exact successful job and selected check identity; canonical rewritten
policy-check URLs remain valid only when exactly one successful matching policy run exists. Focused
fixtures cover each finding and the earlier CHG-0076 URL-rewrite compatibility case.

Second review trigger: two newer findings showed that an earlier archive edge could preserve only
its parent/tree labels while altering moved bytes, and that a newer malformed successful policy
publication could poison an older authenticated success. The archive matcher now reconstructs the
complete expected manifest, canonical ownership, acceptance/closing/review/finalization digests,
bounded sequence history, and exact Git payloads before traversal. It rejects altered, omitted,
extra, forged, or self-reviewed evidence while accepting five real workflow-v2 archive shapes.
Policy publication fallback authenticates each success under an eight-candidate, 30-second bound.

Third review trigger: valid acceptance manifests may contain `non_file` directory inputs, while the
archive matcher originally enumerated only recursive file entries. It also reconstructed sequence
history with a private 256-commit limit instead of the canonical configured limit. Durable invariant:
archive traversal must authenticate tracked directory objects as zero-mode, empty-payload non-file
entries without adding directory nodes to file discovery, and every bounded history query must use
the committed lifecycle limit with a one-entry overflow sentinel. Focused tests cover a real directory
manifest entry, 257 sequence updates, overflow, and malformed configured bounds.
