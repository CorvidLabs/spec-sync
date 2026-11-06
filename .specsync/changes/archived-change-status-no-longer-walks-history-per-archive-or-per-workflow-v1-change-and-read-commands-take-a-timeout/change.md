---
id: archived-change-status-no-longer-walks-history-per-archive-or-per-workflow-v1-change-and-read-commands-take-a-timeout
state: implementing
type: feature
base_commit: 560af9f10e608ce4c506612b0d1d2ae13378807d
---

# Archived change status no longer walks history per archive or per workflow-v1 change, and read commands take a timeout that names the phase and how to resume

## Intent

Archived change status no longer walks history per archive or per workflow-v1 change, and read commands take a timeout that names the phase and how to resume

## Affected Canonical Specs

- `change`
- `cmd_change`
- `cli_args`
- `git_utils`

## Acceptance Criteria

- A warm change status of an archived change spawns fewer than 3 more git processes when the project holds 6 more archived workflow-v1 changes (warm_archived_status_git_processes_do_not_grow_with_legacy_changes, failing first: 94 -> 106 on main)
- A cold archive-introduction index build spawns fewer than 3 more git processes after 4 more archive-introducing commits (one diff-tree --stdin), and an equivalence test pins the per-commit result
- Prefetch workers start from the command repository facts in a read-only seeded scope with no batch reader, so a cold archived status read adds at most 150 git processes for 4 more archives (was 250)
- Whether a workflow-v1 change existed at the trusted pre-v2 cutoff is decided from the cutoff commit tree and kept in the persistent read cache keyed by that commit; existing refusals are unchanged and an equivalence test pins the verdict
- change status, show, list and ship-status take --timeout <secs> (or SPECSYNC_TIMEOUT); on expiry they exit 124, write nothing, flush completed verdicts, and print the phase, progress and exact resume command, in text and JSON
- On demand (fledge run bench-archived-reads, with workflow-v1 changes in the fixture) the warm archived status is under 2 s on an unloaded Mac; cold and warm times on this repository are recorded before and after; the cold under-5 s budget is measured and reported, and any remaining gap goes to a named follow-up
- No gate, bound, timeout default or approval rule is weakened, and no existing assertion is loosened

## No-spec Rationale

Not applicable
