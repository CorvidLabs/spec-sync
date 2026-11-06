---
change: make-the-documented-workflow-tests-run-the-local-verify-step-and-require-it-before-the-scoped-review-906
artifact: docs
---

# Docs

- `site/src/content/docs/index.md`, Quick Start: after `specsync change check add-auth --commit`, add `# run your local verify lane; CI does not test a tip whose change is still open`. That puts the landing page's closeout in the README's order: verify, then the implementation review, then archive, then push.
- `tests/integration/documented_workflows.rs`: the verify-lane rule (design in the commit 3758d970 message and in the `follow` doc comment).
- CHANGELOG.md: an Unreleased / Fixed entry.
- Tasks for the build: the landing-page line, the CHANGELOG entry, then the documented-workflow tests, `fledge lanes run verify` on rust 1.89.0 and `specsync check --strict`. These are listed here because this change selected no tasks artifact. Human gates: Leif's scope approval (queued) and his implementation review.
