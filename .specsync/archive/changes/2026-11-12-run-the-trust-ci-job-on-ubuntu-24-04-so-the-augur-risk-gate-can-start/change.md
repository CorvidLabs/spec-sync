---
id: run-the-trust-ci-job-on-ubuntu-24-04-so-the-augur-risk-gate-can-start
state: archived
type: bug_fix
base_commit: b2ec548ff9fba5f9308a3c7fc61cc853ecd3d0dd
---

# Run the trust CI job on ubuntu-24.04 so the augur risk gate can start

## Intent

Run the trust CI job on ubuntu-24.04 so the augur risk gate can start

## Affected Canonical Specs

- None

## Acceptance Criteria

- The trust CI job runs on an ubuntu-24.04 runner, so the CorvidLabs Trust gate's augur risk step starts (no libxml2.so.2 exit-127), while every other job's runner is unchanged; the workflow comments name CorvidLabs/augur#59 as the reason and the revert condition

## No-spec Rationale

CI runner pin only; no canonical spec changes. ubuntu-latest is rolling to 26.04, whose libxml2 has SONAME libxml2.so.16, and the prebuilt augur binary the Trust gate runs needs libxml2.so.2 (CorvidLabs/augur#59)
