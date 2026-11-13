---
id: the-archive-check-sees-the-default-branch-s-history-so-it-never-tells-a-stale-copy-of-an-upstream-archive-to-ship
state: approved
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# The Archive check sees the default branch's history, so it never tells a stale copy of an upstream archive to ship

## Intent

The Archive check sees the default branch's history, so it never tells a stale copy of an upstream archive to ship

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- the Archive jobs in ci.yml and trust.yml check out full history before the archive gate, and test-required-ci-gate.py fails when a job that runs the gate on the repository checks out a single commit; the archive gate script itself still never fetches

## No-spec Rationale

REQ-cmd-change-029 says archive-readiness.sh, which the Archive check runs, SHALL NOT name change ship for a copy the remote default already archived; the Archive jobs checked out one commit, so the script could not see the default branch and still named ship (#965)
