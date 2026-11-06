---
id: a-cold-archived-change-status-answers-every-archive-s-workflow-version-history-from-one-history-walk-and-the-command-s
state: draft
type: feature
base_commit: 0623412fe1e3cd6e1cc636efb097d100d468a631
---

# A cold archived change status answers every archive's workflow-version history from one history walk and the command's batch reader

## Intent

A cold archived change status answers every archive's workflow-version history from one history walk and the command's batch reader

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- A cold change status of an archived change spawns fewer than 3 more Git processes when the project holds 4 more archives (cold_archived_status_git_processes_do_not_grow_with_archives, committed failing first in 0623412f: 99 with 2 archives, 234 with 6)
- Inside a change read scope, every archived change's workflow-version history is answered from one lifecycle-state history walk per command: candidate state paths, oldest-first state-path history and each commit's parents are derived in memory instead of four Git walks per archive
- The state.json reads and path-exists checks behind that validation go through the command's one object batch reader on the command's thread; a listed blob that cannot be read still refuses and is never cached as absent
- Every archive is still fully validated, so authenticated-history keeps its meaning: equivalence tests show the one-walk verdict equals the per-archive verdict, including the same first refusal and message, for merges, a moved or re-archived package, a deleted creation anchor, a changed identity, a non-canonical archive path and the per-archive history bound
- When the walk cannot answer (over its bound, over the output limit, or a failed Git call) each archive falls back to today's per-archive check, so no new refusal is introduced
- The read deadline (REQ-change-137) still stops the validation between archives with phase and progress, and a warm archived status still spawns no more processes than today
- On demand (fledge run bench-archived-reads) a cold archived status is under 5 s on an unloaded Mac and warm stays under 2 s; cold and warm times on this repository are recorded before and after
- No gate, bound, timeout default or approval rule is weakened, and no existing assertion is loosened

## No-spec Rationale

Not applicable
