---
id: in-a-shallow-clone-name-git-fetch-unshallow-origin-in-every-lifecycle-history-refusal-and-refuse-change-new-before
state: approved
type: bug_fix
base_commit: ed258f58d734be7c4c0ac092ddbc934f492630d3
---

# In a shallow clone name git fetch --unshallow origin in every lifecycle-history refusal and refuse change new before writing

## Intent

In a shallow clone name git fetch --unshallow origin in every lifecycle-history refusal and refuse change new before writing

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- (1) In a depth-1 and a depth-50 clone of an adopted project, change new, change status, change check and change audit refuse, saying the clone is shallow and naming git fetch --unshallow origin; after that fetch they behave as in a full clone. (2) A refused change new leaves no .specsync/changes/<id>/. (3) In a full clone a baseline whose cutoff is not an ancestor keeps today's message, and a corrupt correction ledger keeps its own. (4) The tester's two shapes (a depth-1 clone with no origin ref; a shallow clone with origin/main but not the merge-base) name the fetch.

## No-spec Rationale

Not applicable
