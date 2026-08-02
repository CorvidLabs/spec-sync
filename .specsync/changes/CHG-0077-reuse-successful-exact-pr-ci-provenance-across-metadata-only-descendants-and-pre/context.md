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
manifest entry and all tracked descendants, omitted-descendant and forged-missing attacks, 257
sequence updates, overflow, malformed configured bounds, and broad `.specsync` scopes that exclude
volatile lifecycle workspaces while retaining the legacy baseline.

CI reproduction trigger: the 257-update sequence fixture crossed Git's loose-object threshold on
Ubuntu and background auto-maintenance raced the still-running temporary repository, making a valid
commit intermittently unreadable. Durable invariant: stress fixtures that deliberately create many
Git objects must disable automatic garbage collection and maintenance inside their isolated test
repository. The focused fixture configures both controls before creating sequence history; product
repositories and production behavior are unchanged.

Fourth review trigger: archive reconstruction assumed review evidence was always present before
finalization, treated every tracked file in a spec directory as module-owned, and ignored audited
owner corrections. Those assumptions rejected valid workflow-v2 archives or reconstructed a
different manifest from the native lifecycle. Durable invariant: the reusable-check verifier must
mirror the native finalizer's generated review pair, canonical companion allowlist, exact-delivery
fallback, and signed owner-correction ledger. Focused fixtures cover finalization-generated review
evidence, a noncanonical `retired.md` spec companion, and a corrected canonical co-owner.

Adversarial follow-up trigger: accepting correction fields structurally was insufficient because an
out-of-scope, duplicate, reserved, or non-owning correction could still manufacture ownership, and a
finalization-generated review ledger may contain a prior block before its final pass. Root cause:
the helper recognized the artifact shapes without replaying all native semantic validators. Durable
invariant: correction records must satisfy the native bounds, canonical identity, original-scope,
unique-pair, unaffected-module, regular-blob, and canonical-source ownership rules; every generated
review attempt must be valid and independent, with the projection equal to the final attempt.
Focused regressions cover malformed correction classes and a valid block-to-pass generated ledger.

Final review trigger: Git stores symlinks as blobs, so a blob-only correction check could treat a
symlink or a source-looking file under docs/tests as canonical production ownership. Durable
invariant: corrected paths must use a regular-file mode, a supported source extension, a configured
source directory, and no governed test/fixture segment; canonical specs must also be regular blobs.
The regression fixture covers both symlink and non-production source claims.

Config-review trigger: accepting JSON and TOML source-directory spellings interchangeably could
honor a key that native SpecSync ignores, while assuming `src` when native performs auto-detection
could assign a different ownership boundary. Durable invariant: historical ownership reads only the
format-specific committed key (`source_dirs` for TOML, `sourceDirs` for JSON) and fails closed when
explicit committed source roots are unavailable. Fully signed negative manifests ensure the
symlink/non-production regressions exercise those guards rather than path-set mismatch.

Input-authentication trigger: Git can store configuration and registry paths as symlinks, and the
first historical parser accepted their blob payloads even though native path loading would not treat
a dangling link as the same file. Registry resolution also bypassed native name/mapping semantics and
the configured `specs_dir` fallback. Durable invariant: historical config/registry inputs must be
regular blobs; mapped registries require a nonempty consistent identity, unique supported mapping
shapes, and safe paths when resolved; unmapped modules use the committed configured specs directory.
Fixtures cover dangling config/registry links, nameless mapped registries, and custom-spec fallback.
