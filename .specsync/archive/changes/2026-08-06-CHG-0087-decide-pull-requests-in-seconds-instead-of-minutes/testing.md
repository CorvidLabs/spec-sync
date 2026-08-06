---
change: CHG-0087-decide-pull-requests-in-seconds-instead-of-minutes
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-github-020` | The preflight script was extracted from the workflow with PyYAML and run under GitHub Actions' literal default invocation, `bash --noprofile --norc -eo pipefail`, against seven repository states: no changes directory, evidence commit that is an ancestor of HEAD, null commit, unknown commit, malformed JSON, non-object JSON, and a commit orphaned by a squash merge. It stays silent and exits 0 for the six benign states and fails with a named path for the orphaned one. |

## Manual verification

Measured on pull request #504, whose active change had orphaned evidence:

| job | time | result |
|---|---|---|
| SpecSync implementation ready | 4s | fail |
| trust | 9m31s | fail |
| test | 3m56s | pass |
| audit | 3m08s | pass |

The verdict existed after 4 seconds; roughly 17 minutes of runner time was spent
reaching it again.

## Notes

An earlier draft relied on `set -uo pipefail` inside the step, which cannot
cancel the `-e` GitHub has already applied. A `verification.json` containing
`null` then aborted the job silently, with no annotation, blocking every
subsequent pull request until someone deleted the file. Found by adversarial
review; fixed with `shell: bash {0}` plus a reader that cannot raise. The
original six local tests missed it because they ran under plain `bash`.
