---
id: the-release-tests-exercise-the-race-guards-in-the-shipped-profile
state: draft
type: bug_fix
base_commit: baf6e45509ac1fd4d5cbedfdf87d4dc17805a195
---

# The release tests exercise the race guards in the shipped profile

## Intent

the release tests exercise the race guards in the shipped profile

## Affected Canonical Specs

- `cmd_generate`
- `mcp`
- `validator`

## Acceptance Criteria

- cargo test --release runs a unit test for each of verify_coverage_project_root, RetainedGenerateRoot::verify_public_path and ConfinedReadRoot::revalidate_before_success that replaces the root after it is retained and asserts the guard refuses; fledge lanes run release-candidate runs them under --release (CI-11)

## No-spec Rationale

Adds release-runnable unit tests for three existing TOCTOU guards and a release-candidate lane step; no guard behavior or contract changes
