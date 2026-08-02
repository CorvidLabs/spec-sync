---
change: CHG-0077-reuse-successful-exact-pr-ci-provenance-across-metadata-only-descendants-and-pre
artifact: docs
---

# Docs

- Update the CI confidence guide only if the implementation changes its current user guidance.
- Canonical behavior belongs in `specs/github/requirements.md`, `github.spec.md`, `context.md`, and
  `testing.md`; transient run logs remain in hosted checks and the change package.
- Do not add `change ship` or `change ship-status`; the existing `change status`/`finalize` workflow
  remains the single user path.
