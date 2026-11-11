---
change: journal-error-names-its-file-and-recovery
artifact: context
---

# Context

Issue #433 (part 3 + remediation strings). A corrupt or leftover
`.specsync/change-transaction.json` bricks every write with a raw serde error
(`invalid transaction journal: expected value at line 1 column 1`) — no file
path, no recovery hint. And the stale-evidence refusals print
`specsync change reopen <id>` with no `--actor`/`--reason`, so the pasted
command fails verbatim (`doctor` already prints the complete form).

The journal records an interrupted multi-file write; a corrupt one cannot be
replayed. The recovery is to inspect it and set it aside, after checking no
write was in flight. The fix names the path and that recovery in the error,
and appends the required flags to every reopen remediation.
