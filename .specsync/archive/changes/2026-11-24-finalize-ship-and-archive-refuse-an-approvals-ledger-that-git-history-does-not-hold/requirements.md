---
change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
artifact: requirements
---

# Requirements

Two requirements are added, one per affected module. Their text is in the deltas and is the text Leif
approved as written.

- ADDED REQ-change-166 (`deltas/change.md`): `change finalize`, `change ship` and `change archive`
  archive only an approvals ledger that Git history holds. The working copy equals the version at
  `HEAD`, apart from the closing records an accepted change's accepting step wrote; every committed
  version along `HEAD`'s first-parent history extends the one before it, relay-blind and with the two
  digest members `specsync migrate 5.0` adds. Each refusal names the file and the way out.
- ADDED REQ-cmd-change-committed-ledger-001 (`deltas/cmd_change.md`): `change ship-status` lists that refusal as a
  blocker and does not report the change ready; `change ship --dry-run` refuses with it.

Intent: CHANGE-16, CHANGE-16.a, CHANGE-16.b and CHANGE-16.c (`hi/change.md`).

No existing requirement changes.
