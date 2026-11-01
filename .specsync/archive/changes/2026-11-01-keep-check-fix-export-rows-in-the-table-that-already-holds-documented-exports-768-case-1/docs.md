---
change: keep-check-fix-export-rows-in-the-table-that-already-holds-documented-exports-768-case-1
artifact: docs
---

# Docs

- `specs/cmd_check/requirements.md`: REQ-cmd-check-017 added and REQ-cmd-check-015 reworded, by
  materialization of `deltas/cmd_check.md`.
- `specs/cmd_check/cmd_check.spec.md`: version and Change Log written by materialization only, no
  hand bump.
- `specs/cmd_check/tasks.md`: #768 case 2 recorded as an open task with its reason.
- `specs/cmd_check/cmd_check.3md`: a lesson on a later plane: a `Name` header does not mark an
  export table; the table that already holds documented rows does.
- `specs/cmd_check/testing.md`: the three `fix_768_` tests listed.
- No site or CLI help change: `check --fix` keeps its flags and output; only row placement in
  multi-table blocks changes. The CHANGELOG entry belongs to the release that ships it.
