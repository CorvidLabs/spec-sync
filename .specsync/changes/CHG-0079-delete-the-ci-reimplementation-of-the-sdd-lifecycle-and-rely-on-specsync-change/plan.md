---
change: CHG-0079-delete-the-ci-reimplementation-of-the-sdd-lifecycle-and-rely-on-specsync-change
artifact: plan
---

# Plan

1. Enumerate every assertion made by the machinery proposed for deletion and classify each as
   covered-by-SpecSync or dropped-deliberately. Record the result in `research.md`.
2. Confirm no product source depends on a file being deleted. Retain anything that is read by
   `src/`.
3. Delete the lifecycle reimplementation.
4. Remove the `ci.yml` jobs and `trust.yml` steps that consumed it.
5. Add `specsync change audit --strict` to `spec-check`.
6. Reduce `ci-gate` to one aggregate context with no archive branch and no forced failure.
7. Update `specs/github` so the canonical spec describes the product as the lifecycle authority.
8. Re-run every retained validator and the full Rust suite.

## Out of scope

`classify-ci-paths.sh` is retained. Replacing it with native `paths:` filters would rewrite every
job condition in the same change that deletes 7,257 lines, and the two should not be reviewed
together.

Release validation is retained unchanged. `validate-release-candidate.py` is large, but immutable
release-candidate binding is the one piece of this machinery that earns its cost, and it is not part
of the lifecycle duplication.

Enabling required status checks and code-owner review is a repository setting and follows this
change rather than accompanying it.
