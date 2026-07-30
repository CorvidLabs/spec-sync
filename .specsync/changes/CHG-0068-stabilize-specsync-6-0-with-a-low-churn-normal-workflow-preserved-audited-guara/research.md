---
change: CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara
artifact: research
---

# Research

## Evidence

- PR #455 passes implementation, unit, integration, and coverage checks; `spec-check` fails because
  no active lifecycle workspace covers its changed schema paths. The required gate only propagates
  that failure.
- CorvidLabs/rune PR #23 is a valid 5.2 post-merge cleanup for eight changes, but it modifies 103
  archive files and repeats review/CI despite changing no runtime or canonical contract.
- PR #471 has fresh successful verification and is intentionally frozen as a separate history track.
- PR #462 is a conflicting monolithic lifecycle draft and is excluded.

## Compatibility conclusions

- Existing persisted two-approval records remain byte-compatible historical evidence.
- New 6.0 records use one approval and the same active/archive paths for every change.
- Strict is a validator set selected by `--strict`, policy, or release/security classification; it
  is not persisted as a lifecycle mode and cannot fork transitions or layout.
- `change finalize` mutates lifecycle and canonical files but never calls a GitHub merge API.
- The archive-only lane is a positive validator, not a broad path-based test skip.
- Parent required-check and scoped-review evidence is bound to the implementation commit so the
  metadata-only child does not repeat expensive work.

## Threat model

The finalizer must reject stale approval, failed or missing parent checks, missing scoped review,
unapproved paths, delivery-tree changes, digest mismatch, replay under another PR, invalid archive
ownership, and release attempts with missing merge binding. Pull-request-controlled input cannot
claim green parent checks or expand the archive-only allowlist.

## Investigation findings

- `ci/test-isolation/host-marker-leak`: hosted failures across tests, coverage, spec-check, and trust
  shared one nested integration fixture. The child command removed `CI` but inherited
  `GITHUB_ACTIONS`, so it correctly entered hosted-only validation in a local-test sandbox. The
  invariant is to clear the complete recognized marker set when a fixture asserts local behavior;
  the inherited-marker regression now exercises that exact environment.
- `ci/review-freshness/stale-suppression`: native finalization was stricter than hosted routing.
  Reuse now has one parity rule—schema, verdict, reviewer independence, execution binding, and
  commit-by-commit freshness must all agree—and the classifier fails closed when Git history cannot
  prove them.
- `github/finalize-history/unbounded-query`: commit-by-commit checking closed the net-diff replay
  gap but initially introduced an unbounded hosted query. The corrected design caps command time,
  output bytes, descendants, and parents while still checking every accepted edge.
- `change/adoption-anchor/unavailable-replay`: a compile-time allowlist is not sufficient when the
  historical object check is optional. The exception now requires the exact commit, base parent,
  approvals blob, event, projection, and classification bytes; shallow or foreign replay fails
  closed with a full-history remediation.
- `change/review/self-attested-identity`: local identity strings cannot authenticate independence.
  Review records therefore preserve claimed identity and all pass/block attempts, while the
  authoritative merge proof is the official GitHub Actions scoped-review check bound to the same
  PR, run, and implementation parent.
- `change/finalization/unreachable-implementation`: commit-object reachability is not stable across
  squash/rebase integration, including intermediate block→pass review commits. Reachable commits
  retain strict one-record appends; only a terminal v2 archive introduction with matching accepted
  state, finalization, review digest, and clean subtree may authenticate collapsed history.
- `change/archive/partial-terminal-publication`: two individually atomic writes are not an atomic
  terminal transition. Reusing the lifecycle transaction journal makes interruption recovery occur
  before state dispatch rather than trying to infer a mixed state afterward.
- `change/review/block-erasure`: checking only `review.json` and the ledger tail allowed a later
  pass to conceal an already-committed block. Review children now append exactly one attempt, and
  finalization must preserve the parent ledger byte-for-byte.
- `github/review/block-pass-deadlock`: a blocking review child cannot supply a green review check
  to its pass child. Unchanged remediation preserves the original reviewed implementation identity;
  the pass child proves bounded metadata-only ancestry, reuses that ancestor’s implementation and
  trust results, and becomes the new successful scoped-review check itself.
- `github/workflow/heredoc-import-scope`: embedded workflow programs are separate Python scopes.
  Required bounded-query imports must be asserted within the archive validator itself, not merely
  somewhere in the workflow file.
- `change/workflow-version/downgrade`: serde compatibility defaulted an omitted workflow version to
  v1, so the routing field was not an identity anchor. New records now carry an immutable origin;
  readers validate every parent edge and retain the result in the invocation snapshot so a
  downgrade-and-revert remains visible without multiplying Git scans.
- `change/archive/introduction-rewrite`: structural discovery is useful after squash integration but
  cannot alone authenticate force-rewritten history. Native checks now require one non-root,
  every-parent-absent introduction and an unchanged tree; the post-merge check and release gate bind
  and reconstruct the source introduction, finalization digest, merged commit, and archive tree.
- `change/transaction/torn-journal`: a canonical journal can tear into valid-looking partial JSON,
  and `.ok()` on backup reads conflates absence with I/O or UTF-8 failure. A versioned count/digest
  envelope, durable same-directory replacement, parent-directory sync, exact read errors, and
  idempotent recovery close those crash windows.
- `github/trust/pr-head-policy`: GitHub Actions app identity plus workflow path is insufficient when
  the path is loaded from a PR head. The merged 6.0 bootstrap introduces a read-only
  `pull_request_target` guard whose exact SHA is pinned by the required-workflow ruleset; it fetches
  candidate objects without checkout or execution, blocks policy-file changes, and publishes a
  revision/PR/head-bound check consumed by review, finalization, and post-merge validation.
- `github/archive/fork-publication`: a closed fork PR cannot rely on a write-capable
  `pull_request` token. The archive publisher now runs from the immutable base workflow under
  `pull_request_target`, never checks out PR-controlled content, and fetches exact head/merge
  identities only into the object database before publishing the merge-bound check and comment.
- `github/archive/unbounded-release-history`: independently reconstructing the unique archive
  introduction duplicated uncapped `git log`/`rev-list` calls in merge and release validators.
  A protected shared verifier now limits reachable commits, parents, time, and streamed output,
  proves the archive subtree was never rewritten, and the release validator bounds its other Git
  queries with the same committed policy.
- `github/trust/bootstrap-self-authorization`: candidate-supplied file existence is not a trust
  anchor. The only missing-base-guard exception is now a one-time migration identity frozen to the
  SpecSync repository, PR #480, exact base and refs, descendant relation, and newly added policy
  files; arbitrary repositories, PRs, bases, branches, and pre-existing substitutions fail.
- `change/workflow-v1/first-introduction`: every-parent transition validation has no parent edge for
  a record's first reachable commit. A project-level workflow-v2 baseline supplies the missing
  trusted cutoff: genuine v1 IDs must exist there, while a new record defaulted to v1 by removing
  both fields cannot reach status, legacy accept/archive, or a false-green global check.
- `github/trust/check-production-surface`: protecting selected workflow filenames still lets a new
  workflow or local Action produce a colliding required context. The base guard now treats the
  complete workflow/local-Action namespace as protected and allows optimization only when it is
  unchanged.
- `change/workflow-baseline/squash-parent`: a branch-tip cutoff authenticates local introduction
  but disappears when GitHub squash-merges the branch. Baseline creation now selects the remote
  comparison-base merge point when available, validation requires it to be an ancestor of the
  introduction's first parent, and the one PR #480 bootstrap pins the exact canonical baseline
  bytes to its frozen base.
- `github/trust/local-action-descendant`: an end-anchored directory alternative did not match local
  Action descendants, and rename detection could collapse a protected deletion into an
  unprotected destination. The policy now uses the descendant form plus `--no-renames`; a real Git
  fixture proves add, modify, delete, and rename behavior.
- `change/workflow-v1/explicit-origin`: version-1 policy records can legitimately carry
  `workflow_origin_version: 1`. Cutoff eligibility accepts that exact anchored representation as
  well as the older omitted field, while continuing to reject new first-reachable v1 identities.
- `github/archive/mutable-action-ref`: immutable workflow source does not help if a privileged
  executable dependency uses a moving tag. The post-merge checkout is now pinned to the same full
  Action commit used by the trust workflow.
- `change/status/missing-history-stderr`: the private sandbox retains valid exact legacy evidence
  whose original base objects are no longer available. Boolean `merge-base` probes returned false
  correctly but inherited Git's fatal stderr. Suppressing only those child diagnostics preserves
  fail-closed results while keeping guided text and structured output free of raw implementation
  noise.
- `change/sequence-collision/historical-owner-substitution`: the sandbox's three immutable
  `CHG-0001` collision members all signed the same committed collision-owner sequence ledger.
  After a workflow-v2 draft advanced the current claim, per-record synthetic reconstruction
  replaced that owner with each record's own ID and incorrectly staled two records. Reconstruction
  now performs one bounded, cached history lookup per sequence and reuses exact canonical committed
  bytes only when their same-sequence collision explicitly names the record. Ordinary records and
  post-acceptance collision acknowledgements retain successor-aware synthetic reconstruction.
- `github/trust/root-action-manifest`: the root `action.yml` is executable candidate content because
  CI invokes `uses: ./`, but the initial guard covered only workflows and `.github/actions/**`.
  Root `action.yml` and `action.yaml` now share the protected Action-definition boundary, with
  rename-disabled A/M/D/rename Git fixtures proving the exact workflow regex.
- `change/workflow-history/cross-date-rearchive`: immutable workflow identity can move through more
  than one dated archive path across archive, reopen, and rearchive. Inspecting only the active and
  current archive path converted the prior archive deletion into a false anchor-loss error.
  Validation now uses bounded history to discover every canonical
  `YYYY-MM-DD-<id>/state.json` path for the exact ID and evaluates transitions across their union.
- `github/trust/path-record-boundary`: converting `git diff -z` output to newline-separated text
  destroyed the security boundary between filenames. A workflow filename containing an embedded
  newline split into two lines that each missed the protected regex. The base-controlled guard now
  splits raw bytes only on NUL and applies one full-match byte regex per path; its real Git fixture
  includes the embedded-newline workflow name.
- `github/archive/rewrite-restore`: equal introduction and final archive trees do not prove
  immutable intervening history. The shared verifier now enumerates every bounded path-touching
  commit, bounds each parent list, and requires the introduction tree at the commit and every
  readable parent; a rewrite followed by exact restoration remains rejected.
- `change/workflow-baseline/rewrite-restore`: the project adoption baseline had the same net-diff
  gap. Native validation now checks exact introduction bytes at every bounded touching commit and
  readable parent, so a restored HEAD cannot conceal a temporary cutoff rewrite.
