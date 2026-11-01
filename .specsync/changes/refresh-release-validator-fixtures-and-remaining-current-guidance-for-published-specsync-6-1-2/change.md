---
id: refresh-release-validator-fixtures-and-remaining-current-guidance-for-published-specsync-6-1-2
state: approved
type: documentation
base_commit: 94585d49d549c581db498eca25899856e27fa43c
---

# Refresh release-validator fixtures and remaining current guidance for published SpecSync 6.1.2

## Intent

Refresh release-validator fixtures and remaining current guidance for published SpecSync 6.1.2

## Affected Canonical Specs

- None

## Acceptance Criteria

- In .github/scripts/test-validate-release-candidate.py, the two published-default tests that anchored on the old @main archive-gate example match the published docs: test_archive_gate_main_ref_expires_once_a_newer_release_is_published first rewrites the @v6.1.2 archive-gate example to @main and still requires the newer-release rejection, and test_postpublication_default_requires_deliberate_advance asserts the docs already pin @v6.1.2 and that the validator passes. No other test and no validator logic changes; all 66 tests pass.
- Current guidance stops describing 6.1.2 as unpublished or pinning 6.1.1: site/src/content/docs/quickstart.md (the GitHub Action candidate note), site/src/content/examples/polyglot.mdx and ci-gate.mdx (the candidate notes), SECURITY.md (the CI pinning bullet, including the main wrapper default now 6.1.2), and the github-action.md manual-CI curl download moves from v6.1.1 to v6.1.2.
- MIGRATION.md current 5.x to 6 guidance (binary pin and install tag, Homebrew line, Action pins, version check comment and the explicit Action pin paragraph) moves from 6.1.1 to 6.1.2; historical 5.0 and release-specific history text is unchanged.
- validate-release-version.py, validate-workflow-runtime-pins.py and test-validate-release-candidate.py pass, change audit passes with both active changes, and the pre-push gate passes. No runtime source, canonical spec, dependency, workflow, tag or release changes.

## No-spec Rationale

Validator test fixtures and current user guidance only; no runtime, canonical API, Action default, validator logic, risk or provenance policy changes.
