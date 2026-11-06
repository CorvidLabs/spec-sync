---
id: archived-change-status-no-longer-walks-history-per-archive-or-per-workflow-v1-change-and-read-commands-take-a-timeout
state: draft
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

## Acceptance Criteria

- A warm change status of an archived change spawns fewer than 3 more git processes when the project holds 6 more workflow-v1 changes (warm_archived_status_git_processes_do_not_grow_with_legacy_changes, failing first)
- A cold change status of an archived change builds the archive-introduction index from one git log walk, so its git processes do not grow with the number of archive-introducing commits
- Whether a workflow-v1 change existed at the trusted pre-v2 cutoff is decided from the cutoff commit tree and kept in the persistent read cache keyed by that commit; every existing refusal (hidden downgrade, absent at cutoff, tamper, race) is unchanged and new equivalence tests pin it
- change status, show, list and ship-status take --timeout <secs> (or SPECSYNC_TIMEOUT); on expiry they exit 124, write nothing, flush completed verdicts to the read cache, and print the phase, progress and the exact resume command, in text and JSON
- On demand (fledge run bench-archived-reads, with workflow-v1 changes in the fixture) an archived status takes under 2 s warm and under 5 s cold on an unloaded Mac; on this repository the before and after times are recorded
- No gate, bound, timeout default or approval rule is weakened, and no existing assertion is loosened

## No-spec Rationale

Not applicable
