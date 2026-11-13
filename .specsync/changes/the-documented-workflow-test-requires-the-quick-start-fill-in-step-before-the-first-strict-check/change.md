---
id: the-documented-workflow-test-requires-the-quick-start-fill-in-step-before-the-first-strict-check
state: approved
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# The documented-workflow test requires the Quick Start fill-in step before the first strict check

## Intent

The documented-workflow test requires the Quick Start fill-in step before the first strict check

## Affected Canonical Specs

- None

## Acceptance Criteria

- The documented-workflow test fails when the landing-page Quick Start's fill-in line comes after the first specsync check --strict, and passes for the page as written (red then green)

## No-spec Rationale

A test-only fix: the documented-workflow test checked the landing-page fill-in step only against add-spec, so a page that told readers to run check --strict on the untouched scaffold would still pass (DOCS-1). No spec or requirement text changes.
