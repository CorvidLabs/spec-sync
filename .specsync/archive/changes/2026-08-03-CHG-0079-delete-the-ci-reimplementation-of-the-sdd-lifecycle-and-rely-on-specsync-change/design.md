---
change: CHG-0079-delete-the-ci-reimplementation-of-the-sdd-lifecycle-and-rely-on-specsync-change
artifact: design
---

# Design

SpecSync is the single authority on lifecycle coherence. CI asks the product; it does not re-derive
the answer from commit topology.

## Removed

- `.github/scripts/reuse-check-from-ancestors.py` and its test suite
- `.github/scripts/verify-archive-introduction.py`
- `.github/scripts/verify-trusted-policy-check.py` and its test suite
- `.github/scripts/test-lifecycle-workflows.sh`
- `.github/workflows/finalize-change.yml`
- `.github/workflows/post-merge-archive.yml`
- `.github/workflows/lifecycle-policy-guard.yml`
- `ci.yml` jobs `archive-integrity` and `scoped-review-reuse`
- The `trust.yml` metadata-child classification and ancestor-reuse steps

## Added

`spec-check` runs `cargo run -- change audit --strict`.

## Changed

`ci-gate` becomes one aggregate context suitable for a required status check. It no longer fails a
green implementation pull request to demand a separate archive-tip commit, and no longer branches on
archive-only classification.

## Evidence binding

Evidence binds to the merge commit rather than to each intermediate tip. GitHub already guarantees
the merge commit's identity and which checks passed on it, so the traversal machinery that
reconstructed this from ancestry is unnecessary rather than broken.

## Policy enforcement

The custom guard is replaced by CODEOWNERS, which already covers these paths via `* @CorvidLabs/humans`.
Enforcement requires enabling "Require review from Code Owners" and a non-zero approving review count
in branch protection. That is a repository setting, not a file in this change.
