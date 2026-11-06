## ADDED

### REQUIREMENT REQ-change-143

`specsync change close-late <id>... --actor <human> --reason <text>` SHALL close, in one step and without re-checking it, a change that merged to the remote default branch but was never archived, and SHALL record that change as closed late and never verified.

Acceptance Criteria
- A change is eligible in any state except `archived` and `closed_late`: draft, approved, implementing, verifying or accepted, including a partial finalize left under `.specsync/changes/` and an `accepted` package already under `.specsync/archive/changes/`. Workflow-v1 and workflow-v2 changes close the same way.
- The package's `state.json`, at its current path, must be present on the remote default reference (`origin/HEAD`, else `origin/main` or `origin/master`). When no remote default resolves, or the package is not on it, the change is refused.
- close-late reads none of the change's delivery inputs, rebuilds no acceptance manifest or acceptance input digest, and runs no verification.
- An active package moves into `.specsync/archive/changes/<date>-<id>/`; a package already in the archive stays where it is. `state.json` takes the terminal state `closed_late`, and `late-close.json` records the schema version, change ID, actor, reason, timestamp, the state it was closed from, its workflow version, the remote default reference and its commit, the commit that added the package and that commit's date, and `verified: false`. Every other file in the package keeps its bytes.
- An empty `--actor` or `--reason` is refused.
- A `closed_late` change never counts as verified: the archive post-move preflight does not authenticate it, it never covers another change's delivery inputs as a successor, `reopen`, `review` and `finalize` refuse it, and `change audit` counts it closed.
- `change status` (text and JSON) and `change ship-status` report the late close: who closed it, when and why, the commit that merged it and that commit's date, that it was never re-checked, and `verified: false`. A change closed from draft is also reported as never approved.
- Several IDs are all checked before anything is written; one refusal writes nothing and names the refused change. `--dry-run` reports each change that would close and why, and writes nothing.
- Every refusal leaves the tree byte for byte as it was.
