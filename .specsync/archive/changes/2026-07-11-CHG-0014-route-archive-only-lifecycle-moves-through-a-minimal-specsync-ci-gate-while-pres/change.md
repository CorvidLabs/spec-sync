---
id: CHG-0014-route-archive-only-lifecycle-moves-through-a-minimal-specsync-ci-gate-while-pres
state: archived
type: operations
base_commit: 58cc99e6c4d585426a3a355843fd672e0f1cd220
---

# Route archive-only lifecycle moves through a minimal SpecSync CI gate while preserving full validation for source, dependency, action, workflow, and release changes

## Intent

Route archive-only lifecycle moves through a minimal SpecSync CI gate while preserving full validation for source, dependency, action, workflow, and release changes

## Affected Canonical Specs

- None

## Acceptance Criteria

- Archive-only moves run SpecSync lifecycle validation without platform tests, coverage, audit, site, extension, or packaged Action jobs.
- Source, dependency, workflow, action, and release changes still run the full matrix.
- Site-only and VS Code-only changes run their relevant product job plus SpecSync validation.
- Skipped jobs remain neutral in the PR summary and the stable aggregate gate.
- Main records attestation only after every selected gate succeeds.
- Classifier behavior is covered by deterministic tests.

## No-spec Rationale

This changes repository CI orchestration only; it does not alter the SpecSync public contract or canonical module behavior.
