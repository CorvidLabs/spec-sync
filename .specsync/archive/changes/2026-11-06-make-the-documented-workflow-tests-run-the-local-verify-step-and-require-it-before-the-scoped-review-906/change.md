---
id: make-the-documented-workflow-tests-run-the-local-verify-step-and-require-it-before-the-scoped-review-906
state: archived
type: documentation
base_commit: 0b209a5c1398b9e9b881ced48a3145b17e8492b9
---

# Make the documented workflow tests run the local verify step and require it before the scoped review (#906)

## Intent

Make the documented workflow tests run the local verify step and require it before the scoped review (#906)

## Affected Canonical Specs

- None

## Acceptance Criteria

- In the README and landing-page workflow tests, a prose step naming the verify lane runs the fixture lane (specsync check --strict), which must pass, on the current tip; change review is allowed only after that ran on the tip it reviews (DOCS-2, #906).
- The tests fail if a documented closeout drops its verify step or puts it before change check --commit or after change review: the README with its verify line removed fails at the review, and the landing page fails at 3758d970 because it has none.
- The landing page closeout gains the verify step after change check --commit, worded like the README (run the local verify lane; CI does not test a tip whose change is still open), and its test passes.
- The landing page's Quick Start tells the reader to fill in the add-spec auth scaffold before the change starts, in one line after add-spec: '# fill in specs/auth/ (the spec and its companions; Spec Format guide)'. The landing-page tests fill it in at that line as a reader would (real sections, status: active, login documented, no scaffold markers), and the fixture lane stays specsync check --strict (Leif, 2026-11-06, option A).
- No assertion of the existing documented-workflow tests is loosened.

## No-spec Rationale

A documentation test and one landing-page line; no module contract, export or requirement changes
