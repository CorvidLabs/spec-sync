---
change: send-an-approved-change-back-to-draft-for-re-scoping
artifact: tasks
---

# Tasks

- [x] Capture Leif's confirmed criteria in hi as CHANGE-12 to CHANGE-12.c.
- [x] Write failing tests for send-back, then implement it.
- [x] Cancel approvals recorded before a send-back in the definition gate.
- [x] Wire `change send-back` in the CLI with text and JSON output.
- [x] Document send-back in the workflow and CLI guides.
- [x] Record requirement evidence in specs/change, specs/cmd_change and specs/cli_args testing.
- [x] Review round (e52f77d): list applied items at send-back and refuse orphaned applied text in check and accept.
- [x] Review round: say "definition approval" in REQ-change-123 and the guides, matching CHANGE-12.b.
- [x] Review round: honour send-backs in the definition approval event resolver, the scoped-review approver lookup and the scope-expansion summary.
- [x] Review round: refuse a ledger whose committed send-backs were removed or edited.
- [x] Review round: finalize test uses a change that owns a spec.
- [x] Review round two (6e9abc9): converge after a send-back so REMOVED and changed ADDED items check again.
- [x] Review round two: list REMOVED items and pre-change text; advise restoring it; refuse a dropped module's Change Log row.
- [x] Review round two: check every committed revision of the send-back history, and report it in the audit.
