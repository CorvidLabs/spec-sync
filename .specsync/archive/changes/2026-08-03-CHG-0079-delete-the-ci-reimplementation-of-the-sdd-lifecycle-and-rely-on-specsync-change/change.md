---
id: CHG-0079-delete-the-ci-reimplementation-of-the-sdd-lifecycle-and-rely-on-specsync-change
state: archived
type: refactor
base_commit: 8c58aa29a7ccf7e369b33ff8244549505360ca6f
---

# Delete the CI reimplementation of the SDD lifecycle and rely on specsync change audit, removing the separate-archive-tip constraint

## Intent

Delete the CI reimplementation of the SDD lifecycle and rely on specsync change audit, removing the separate-archive-tip constraint

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- The CI reimplementation of lifecycle rules is deleted (reuse-check-from-ancestors, verify-archive-introduction, verify-trusted-policy-check, test-lifecycle-workflows, finalize-change.yml, post-merge-archive.yml, lifecycle-policy-guard.yml); ci.yml runs specsync change audit --strict as the single lifecycle authority; the Required CI gate no longer fails a green implementation PR demanding a separate archive-tip commit; trust.yml no longer consumes ancestor reuse; remaining validators (classify-ci-paths, validate-workflow-runtime-pins, validate-release-candidate) still pass; lifecycle-validation-limits.json is retained because src/change.rs reads it

## No-spec Rationale

Not applicable
