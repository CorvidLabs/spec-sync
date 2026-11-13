---
id: the-documented-workflow-test-needs-an-instruction-to-run-the-verify-lane-before-the-human-review
state: draft
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# The documented-workflow test needs an instruction to run the verify lane before the human review

## Intent

The documented-workflow test needs an instruction to run the verify lane before the human review

## Affected Canonical Specs

- None

## Acceptance Criteria

- The documented-workflow test fails when the README's verify-lane line is reworded so it no longer tells the reader to run the lane, and when the human-review line is moved above it; the README and landing page as written pass (red then green)

## No-spec Rationale

A test-only fix: the documented-workflow test counted any prose naming the verify lane as running it, and checked the order only at change review, so a page that no longer taught verify-then-review could still pass (DOCS-2). No spec or requirement text changes.
