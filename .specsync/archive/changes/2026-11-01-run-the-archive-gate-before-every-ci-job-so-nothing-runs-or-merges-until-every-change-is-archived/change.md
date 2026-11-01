---
id: run-the-archive-gate-before-every-ci-job-so-nothing-runs-or-merges-until-every-change-is-archived
state: archived
type: operations
base_commit: 24e29e100930db10e2f9db78bb7bcbf5ab6507e4
---

# Run the archive gate before every CI job so nothing runs or merges until every change is archived

## Intent

Run the archive gate before every CI job so nothing runs or merges until every change is archived

## Affected Canonical Specs

- `github`
- `cmd_change`
- `agents`

## Acceptance Criteria

- A tip with any open change fails the Archive job first in CI and Trust, every other job is skipped, and the required CI gate fails; a tip with every change archived runs the full lane selected from the whole pull request; the Action's require-archived and archive-gate-only inputs run the same gate before downloading SpecSync; ship-status and agent guidance describe verify, review and ship locally, then push the archive tip.

## No-spec Rationale

Not applicable
