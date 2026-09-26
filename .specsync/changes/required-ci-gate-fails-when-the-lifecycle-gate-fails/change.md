---
id: required-ci-gate-fails-when-the-lifecycle-gate-fails
state: implementing
type: bug_fix
base_commit: cddc39e478dcc1f111940a3cfb02134bba9804cc
---

# Required CI gate fails when the lifecycle gate fails

## Intent

Required CI gate fails when the lifecycle gate fails

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- Required CI gate fails whenever Lifecycle preflight or Lifecycle gate fails or is cancelled. implementation-gate (SpecSync implementation ready) needs classify, preflight, lifecycle-gate and every other job that can finish before it, and accepts skipped for a job only when that job's own if: condition, evaluated again over the classify outputs, deselected it; a selected job that was skipped because something it needs did not succeed fails the gate. Full, site-only, VS Code-only, specs/lifecycle-only, archive-only, legacy archive-only and review-only pull requests, pushes to main and workflow_dispatch runs stay green when every selected job succeeds. .github/scripts/test-required-ci-gate.py runs in the validate-action CI job and in the Fledge verify lane; it fails if preflight or lifecycle-gate is missing from implementation-gate.needs, if a job that gates on lifecycle-gate or can otherwise finish before the gate is missing, or if a gate row no longer matches its job's if:, and it simulates every classify path to show the required gate red when any one selected job fails or is cancelled and green otherwise, and reproduces #796 against the pre-fix gate.

## No-spec Rationale

Not applicable
