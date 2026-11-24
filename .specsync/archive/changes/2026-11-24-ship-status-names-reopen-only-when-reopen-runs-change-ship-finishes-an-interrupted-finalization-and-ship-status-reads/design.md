---
change: ship-status-names-reopen-only-when-reopen-runs-change-ship-finishes-an-interrupted-finalization-and-ship-status-reads
artifact: design
---

# Design

- **#922, asked of the change's own evidence.** For an accepted or archived change, ship-status asks the validator that already judges accepted and archived inputs, read-only, whether the change's own accepted evidence is current. Current: the product-tip stage is done. Stale: the product-tip stage and `ship_next` name `specsync change reopen <id> --actor <name> --reason <text>`, never `change check`. `verification.json` unreadable: both name its restore, not reopen.
- **#933, readiness as finalize judges it.** An accepted change is ready to finalize when `change finalize` would resume it: the check finalize itself makes before it resumes (its `finalization.json` and the review it binds), with no blocker. `change ship` then archives it through finalize's resume. An open task still blocks, naming the task and `specsync change ship <id>`.
- **#938, once per report.** `ship_status_report` evaluates the sequence-ledger freeze once and passes it to the lifecycle next action, so the blockers, `ship_next` and the lifecycle next action read the same value.
- **#1038.** One assertion in the reconciled-history test: an archived change whose terminal evidence is `historical-transition-unproven` reads its review as done by finalize.

Rejected: #933's option 2 (name `change finalize` and correct AGENTS.md), because AGENTS.md, REQ-cmd-change-031 and text status already promise that `change ship` works there.
