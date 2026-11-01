---
id: let-the-release-validator-accept-the-action-archive-gate-in-ci-and-docs-before-its-release
state: archived
type: operations
base_commit: a5043451fe1cde489347f036d209dee088292310
---

# Let the release validator accept the Action archive gate in CI and docs before its release

## Intent

Let the release validator accept the Action archive gate in CI and docs before its release

## Affected Canonical Specs

- None

## Acceptance Criteria

- validate-release-version.py finds the packaged Action consumer step by its version input while action-consumer also runs the archive gate; documented steps that set require-archived or archive-gate-only may pin @main only while the published Action default is 6.1.1 or older; every other documented step still must pin the published release; test-validate-release-candidate.py passes and proves the allowance expires.

## No-spec Rationale

Release-version validation tooling only; no canonical spec owns these CI scripts
