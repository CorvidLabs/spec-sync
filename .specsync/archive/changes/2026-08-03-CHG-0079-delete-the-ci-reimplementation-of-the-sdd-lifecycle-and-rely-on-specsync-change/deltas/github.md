## MODIFIED

### REQUIREMENT REQ-github-008

SpecSync SHALL be the single authority on lifecycle coherence in hosted verification. CI SHALL prove
lifecycle correctness by invoking the product rather than by re-deriving lifecycle rules from Git
commit topology, and SHALL NOT require a separate archive-tip commit before merge.

Acceptance Criteria

- The pull-request CI workflow runs `specsync change audit --strict` and fails closed on its result.
- No workflow or script in `.github/` reimplements archive introduction, archive integrity,
  post-merge archive binding, or metadata-descendant provenance.
- A green implementation pull request is mergeable without pushing an additional archive-move commit;
  the required aggregate gate does not fail a passing implementation to demand one.
- Lifecycle rules that CI no longer checks are either proven by `specsync change audit --strict` and
  the Rust suite over `src/change.rs`, or recorded as deliberately dropped.
- Protected-path authorization is enforced by CODEOWNERS and branch protection rather than by a
  repository-specific policy script that has no passing path for any pull request touching it.
