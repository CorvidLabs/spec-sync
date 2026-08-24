# cmd_change lessons-loop delta

## MODIFIED

### SPEC SECTION Invariants

1. JSON output contains no terminal coloring.
2. Domain errors always produce exit code 1.
3. `change check` runs scoped verification for one change only and fails when that verification fails; it does not rewalk archived terminal evidence.
4. `change audit` reports active-workspace and living-spec integrity only and exits non-zero on report errors.
5. `change finalize` requires current verification and scoped-review evidence and performs no provider merge.
6. `change ship-status` decides readiness from evidence CURRENCY — the recorded plan and tree still match what was verified — never from whether the recorded commit is reachable from HEAD. A squash-merge rewrites that commit, so reachability would make a squash-merged change permanently unfinalizable while its evidence is intact.
7. The lessons loop surfaces at each of the three moments a lesson exists: `change new` names every affected module's `specs/<module>/context.md` that holds substantive prose, a FAILED `change check` names where to record what the failure taught, and `finalize` names folding the archived bundle into those specs before the merge. Every surface is a pointer, never a dump, and none can fail a lifecycle command. A passing `change check` says nothing.

### SPEC SECTION Error Cases

| Condition | Behavior |
|-----------|----------|
| Unknown change type | Descriptive error and exit 1 |
| Invalid transition | Current and expected states plus exit 1 |
| Missing actor or reason | Clap or domain validation exits non-zero without lifecycle mutation |
| Current or successor-covered accepted evidence | Reopen reports the shared non-stale reason and exits 1 |
| Missing or mismatched supersede obligation | Command reports the exact predecessor/path/module/digest mismatch and exits 1 without definition mutation |
| Invalid exact owner correction | Command reports the domain rejection and exits 1 without lifecycle mutation |
| Invalid batch owner correction or empty discovery | Command reports the domain rejection and exits 1 without lifecycle mutation |
| Scope approver records the scoped review, or the current verdict is blocking | Command reports the independent-review rejection and finalization remains blocked |
| Invalid correction ledger before answer, depend, or supersede | Command emits the safe integrity diagnostic and leaves lifecycle files unchanged |
| Correction ledger changes after a successful mutation | Command renders the transaction's validated snapshot and does not report a false failure after persistence |
| Affected module has no `context.md`, or it holds only scaffold prompts | Surfacing is skipped for that module and change creation is unaffected |
