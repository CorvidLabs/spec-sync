---
id: teach-archive-first-closeout-in-the-readme-and-run-its-workflow-example-as-a-test-881-d5
state: archived
type: documentation
base_commit: ad50bdef3db58a715877877d2407485bcaed0f32
---

# Teach archive-first closeout in the README and run its workflow example as a test (#881, D5)

## Intent

Teach archive-first closeout in the README and run its workflow example as a test (#881, D5)

## Affected Canonical Specs

- None

## Acceptance Criteria

- Running the README Verified change workflow block as written, with the by-hand steps it names supplied, ends with the change archived, and every line in it that tells the reader to wait for CI comes at a point where the CI Archive gate passes (DOCS-1, DOCS-2).
- Until finalize the Archive gate refuses the open change; each distinct command its refusal names runs successfully in a copy of the project, and the named `change ship` then satisfies the gate.
- The new test fails on the README at ad50bdef, whose wait-for-CI line comes before the scoped review, and passes on the corrected README. It runs under cargo test, and README.md is a full-lane CI path, so CI fails when the example drifts (DOCS-1.a).
- The README closeout says what Chough teaches (chough ec830cfb): the hosted Archive gate refuses while a change is open, so finish locally before waiting on CI, and a push before the review only shows the work early. The no-commit-between-review-and-finalize rule and the human implementation review stay (DOCS-2.a).

## No-spec Rationale

README guidance and an integration test that executes it; no module contract, export or requirement changes
