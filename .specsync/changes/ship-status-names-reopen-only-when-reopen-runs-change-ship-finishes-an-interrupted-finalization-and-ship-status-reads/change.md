---
id: ship-status-names-reopen-only-when-reopen-runs-change-ship-finishes-an-interrupted-finalization-and-ship-status-reads
state: verifying
type: bug_fix
base_commit: a28e30420c12627a384a685cd5d808fc3585b70a
---

# Ship-status names reopen only when reopen runs, change ship finishes an interrupted finalization, and ship-status reads the ledger freeze once per report

## Intent

ship-status names reopen only when reopen runs, change ship finishes an interrupted finalization, and ship-status reads the ledger freeze once per report

## Affected Canonical Specs

- `cmd_change`
- `change`

## Acceptance Criteria

- #922: an archived change whose own accepted evidence is current is not told to reopen after a commit that changes only an unrelated file, its product-tip stage is done, and `change reopen` refuses it; an accepted or archived change whose own accepted evidence is stale is told to reopen in the product-tip stage and in `Next:`, never to run `change check`, and `change reopen` accepts it; an archived change whose `verification.json` cannot be read is told to restore it and is not told to reopen.
- #933: after a finalization that stopped between acceptance and the archive move, `change ship-status <id>` reports the accepted change ready to finalize and `change ship <id>` archives it; with a task reopened in that state, `change ship <id>` refuses naming the task and `specsync change ship <id>`, and once the task is ticked that named command archives the change; an accepted change that `change finalize` would refuse to resume is not reported ready.
- #938: `change ship-status` evaluates the sequence-ledger freeze once for each change it reports, and its blockers, `ship_next` and lifecycle next action are unchanged, with and without a freeze.
- #1038 follow-up: the reconciled-history test asserts that an archived change whose terminal evidence is `historical-transition-unproven` reads its review as done by finalize.

## No-spec Rationale

No requirement text changes. REQ-cmd-change-033 lets the ship-status product-tip stage advertise only a command whose state guard admits the change, and names `change reopen` for an accepted or archived change whose verification is no longer current; `change reopen` refuses a change whose own accepted evidence is still current and one whose `verification.json` it cannot load, so the stage advertised a command that refuses (#922). REQ-cmd-change-031 has the open-task refusal name `specsync change ship <id>` and says running that command archives the change, and text status tells an accepted workflow-v2 change to run `change ship` or `change finalize`; ship readiness accepted only a verifying change, so the named command refused (#933). ship-status evaluated the sequence-ledger freeze twice for each report; evaluating it once changes no output (#938).
