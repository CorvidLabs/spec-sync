## ADDED

### REQUIREMENT REQ-github-005

Maintained GitHub automation SHALL finalize and validate change archives on the originating PR
without repeating the implementation matrix or bypassing merge protections.

Acceptance Criteria

- Required implementation checks and one scoped review for agent-authored work bind the
  implementation parent commit.
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
- Release validation rejects integrated changes lacking valid same-PR finalization and merge binding.
- Workflow permissions are least privilege and fork-controlled input cannot forge parent status,
  review, allowlisted paths, or archive identity.
