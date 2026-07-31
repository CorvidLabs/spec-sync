## ADDED

### REQUIREMENT REQ-github-005

Maintained GitHub automation SHALL finalize and validate change archives on the originating PR
without repeating the implementation matrix or bypassing merge protections.

Acceptance Criteria

- Required implementation checks and one schema-v2 passing scoped review for agent-authored work
  bind the implementation parent commit, execution/workspace digests, append-only attempt history,
  and exact GitHub Actions check provenance.
- The required merge gate remains incomplete until same-PR finalization; a review-metadata-only
  child reuses its parent's product checks and independent review without rerunning either.
- Same-PR finalization produces a child commit containing only exact approved lifecycle/archive
  changes.
- The archive-only lane verifies parent green checks, exact diff classification, unchanged delivery
  tree, archive integrity, bidirectional ownership, and finalization digest, then reports to required
  CI without selecting the full product matrix or scoped reviewer again.
- GitHub branch protection or merge queue performs the merge; SpecSync automation never invokes a
  merge API.
- A lightweight post-merge job may bind actual merge SHA/tree to the archive digest and retry
  transient failures without writing code files.
- Squash/rebase integration preserves an exact archive-subtree anchor for fresh-clone validation
  after the implementation commit becomes unreachable.
- Every bounded archive-path-touching commit and readable parent retains the exact introduction
  subtree, so an intermediate deletion or rewrite cannot be concealed by restoring final bytes.
- Release validation rejects integrated changes lacking valid same-PR finalization and merge binding.
- Workflow permissions are least privilege and fork-controlled input cannot forge parent status,
  review, allowlisted paths, or archive identity.
- Merged-fork archive publication executes only immutable base-controlled workflow code and fetches
  PR identities as Git objects without checking out or executing candidate content; every
  privileged executable Action dependency is pinned to a full commit SHA.
- The trusted policy guard rejects changes to every `.github/workflows/*.yml`,
  `.github/workflows/*.yaml`, root `action.yml`/`action.yaml`, `.github/actions/**` definition, and
  the workflow-v2 baseline. It disables rename detection so protected deletions and moves remain
  visible, preserves NUL filename boundaries, and full-matches each raw Git path independently; its
  initial exception is frozen to one repository, PR, exact base, branch identity, canonical
  exact-base baseline, and required added-file set.
- Workflow-v1 archive moves stay on the historical full-validation path and a v2 parent cannot
  downgrade itself into that route.
- Fork PRs run the same read-only scoped-review analysis while comments/review writes stay disabled.
- Classifier and finalizer history bounds load one committed limits document shared with native
  review freshness validation.
