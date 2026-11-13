---
change: close-changes-that-will-not-ship
artifact: delta
module: cmd_change
---

## ADDED

### REQUIREMENT REQ-cmd-change-037

`specsync change close <id>... --actor <name> --reason <text>` SHALL run `close_changes`, and SHALL
take `--replaced-by <id>` and the relayed-decision flags `--words`, `--where` and `--recorded-by`
as `change approve` does. Text output SHALL print one line per closed change with the state it
was closed from and its package, then the next step: commit the move. `--json` SHALL print
`{"closed": [...]}`. `change status`, `show` and `ship-status` of a closed change SHALL print who
closed it, when and why, what replaces it, and that it was never reviewed, with
`summary.state: closed` in JSON (#1070).

#### Acceptance Criteria

- `change close a b --actor Ada --reason "..." --json` prints both closed changes.
- `change status <id>` of a closed change prints `closed; never reviewed` and the reason, and
  `--json` reports `summary.state` `closed`.
- A relayed decision missing one of its three flags is refused before anything moves.
