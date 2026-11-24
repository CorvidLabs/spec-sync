---
id: give-the-macos-release-candidate-qualification-120-minutes
state: verifying
type: operations
base_commit: b114d09fd1b102ddbb57dd4032769d6c854e5c52
---

# Give the macOS release-candidate qualification 120 minutes

## Intent

Give the macOS release-candidate qualification 120 minutes

## Affected Canonical Specs

- None

## Acceptance Criteria

- The `qualify` job in `.github/workflows/release.yml` has `timeout-minutes: 120`, and its comment says why. The 6.4.2 handoff records that `v6.4.2-rc.1` was cancelled on macOS at 75 minutes with every test passed and that the candidate is cut again. The version validator, the workflow-pin validator and the release-candidate contract tests pass. No product, test or spec file changes.

## No-spec Rationale

A CI job limit and a release handoff note. No module contract, requirement or product behaviour changes.
