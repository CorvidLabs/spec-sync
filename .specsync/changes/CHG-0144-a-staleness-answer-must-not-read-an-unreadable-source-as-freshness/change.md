---
id: CHG-0144-a-staleness-answer-must-not-read-an-unreadable-source-as-freshness
state: implementing
type: bug_fix
base_commit: efa8d70ee29bdb5d16ef8cdb9177096d63e44ee7
---

# A staleness answer must not read an unreadable source as freshness

## Intent

a staleness answer must not read an unreadable source as freshness

## Affected Canonical Specs

- `cmd_stale`
- `cmd_report`
- `cmd_check`
- `scoring`
- `cmd_lifecycle`
- `git_utils`

## Acceptance Criteria

- every command that answers a staleness question refuses to call a spec current when a file it cites no longer exists; a committed deletion is stale regardless of threshold because it measures one commit; a path git never tracked is unmeasurable and fails closed unless enforcement is warn; a spec that measured some files discloses the ones it could not; the all-clear is withheld in every output format when anything went unmeasured; the machine-readable form carries the same distinctions as the human one; report sets its inconclusive flag whenever any module was unmeasured; scoring reports the git half withheld rather than measured at zero and applies no second penalty so the score is unchanged; the lifecycle no-stale guard fails on a deleted cited file; a healthy spec, sub-threshold drift, real drift and a warn project all behave exactly as before

## No-spec Rationale

stale printed All specs are up to date and exited 0 for a spec whose only cited source file had been deleted, while check exited 1 on the same tree; five production sites shared the belief in three disguises
