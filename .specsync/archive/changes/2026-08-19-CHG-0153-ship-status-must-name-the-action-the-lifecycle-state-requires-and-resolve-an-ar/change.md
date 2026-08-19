---
id: CHG-0153-ship-status-must-name-the-action-the-lifecycle-state-requires-and-resolve-an-ar
state: archived
type: feature
base_commit: 81f752c0c0cef613ef2f75740cf645592f51eb39
---

# Ship-status must name the action the lifecycle state requires, and resolve an archived change's evidence

## Intent

ship-status must name the action the lifecycle state requires, and resolve an archived change's evidence

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- change ship-status prints a Next: line that names an action the same binary will accept: at draft it names the interview question rather than change check --commit, at approved it names an action rather than restating a blocker, and at archived it names no further action; ship-status resolves an archived change's verification and scoped review from wherever the change actually lives instead of from a hard-coded active path, so an archived change reports its verification commit and review presence rather than none and missing; a corrupt or unreadable archived verification.json degrades to no evidence rather than failing the command, so an already-damaged repository is not bricked by the fix; and the active-or-archive resolution reuses the existing find_change_dir primitive rather than adding a third workspace-resolution idiom beside change_dir and find_change_dir.

## No-spec Rationale

Not applicable
