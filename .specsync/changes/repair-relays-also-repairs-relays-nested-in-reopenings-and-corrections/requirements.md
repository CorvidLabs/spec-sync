---
change: repair-relays-also-repairs-relays-nested-in-reopenings-and-corrections
artifact: requirements
---

# Requirements

- REQ-change-relay-repair-001 (`deltas/change.md`): the repair reaches each reopening's
  `superseded_approval` in `approvals.json` and both approvals each `corrections.json`
  correction embeds. A nested record is put back from the same place at `HEAD`, or its relay
  alone is removed, and no reopening or correction is removed. One new criterion says so.
- REQ-cmd-change-relay-repair-001 (`deltas/cmd_change.md`): the printed places add
  `approvals.json reopening <n> superseded approval` and
  `corrections.json correction <n> definition approval` or `closing approval`. The JSON repair
  gains `within` for a nested record.
