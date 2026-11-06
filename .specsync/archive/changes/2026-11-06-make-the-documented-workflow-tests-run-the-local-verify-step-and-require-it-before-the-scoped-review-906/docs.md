---
change: make-the-documented-workflow-tests-run-the-local-verify-step-and-require-it-before-the-scoped-review-906
artifact: docs
---

# Docs

- `site/src/content/docs/index.md`, Quick Start: after `specsync change check add-auth --commit`, add `# run your local verify lane; CI does not test a tip whose change is still open`. That puts the landing page's closeout in the README's order: verify, then the implementation review, then archive, then push.
- `site/src/content/docs/index.md`, Quick Start: after `specsync add-spec auth`, add `# fill in specs/auth/ (the spec and its companions; Spec Format guide)` (Leif's wording, option A). A strict verify lane refuses the unfilled scaffold, and the page did not say to fill it in.
- `tests/integration/documented_workflows.rs`: the landing-page fixture fills in the scaffold at that line, as a reader would: the spec and its four companions, with real sections, `status: active`, `login` in Public API and no scaffold markers. It fails if the line is missing or comes before `add-spec auth`. The fixture lane stays `specsync check --strict`.
- `tests/integration/documented_workflows.rs`: the verify-lane rule (design in the commit 3758d970 message and in the `follow` doc comment).
- CHANGELOG.md: an Unreleased / Fixed entry.
- Tasks for the build: the two landing-page lines, the scaffold fill-in in the harness, the CHANGELOG entry, then the documented-workflow tests, `fledge lanes run verify` on rust 1.89.0 and `specsync check --strict`. These are listed here because this change selected no tasks artifact. Human gates: Leif's re-approval of this amended scope, and his implementation review on GitHub.
