---
change: repair-a-damaged-relayed-decision-with-one-command-whether-or-not-it-is-committed
artifact: requirements
---

# Requirements

- REQ-change-relay-repair-001 covers the repair, the refusal wording and the archived and not-JSON refusals. It carries CHANGE-13.e and 13.f.
- REQ-cmd-change-relay-repair-001 covers the command's text and JSON output. It carries CHANGE-13.g.
- REQ-cli-args-relay-repair-001 covers the subcommand grammar.
- REQ-change-relayed-decision-001 (from #900) is modified so its read-refusal criterion names `repair-relays` for an active change and keeps `git checkout HEAD` for an archived one; the two cannot then promise different recoveries.
- The change Public API section adds `RelayRepair`, `RelayRepairReport` and `repair_relayed_decisions` and updates the `recorded_claims` row's recovery wording.

Named requirement IDs are used so parallel rows adding numbered requirements cannot collide with these; the orc registered them.
