---
change: journal-read-errors-name-the-file-and-a-quoted-recovery
artifact: context
---

# Context

Round 2 of #433 (PR #1041), t384's review. Two gaps in round 1: the journal
READ branch (non-UTF-8 bytes, a directory at the path) still returned a bare
error, and the printed `mv` recovery was unquoted, so it failed verbatim in a
project whose path contains a space. Both are the same promise as round 1: a
refusal names the file and a recovery that runs as printed.

Also adopts the reviewer's smaller wording for REQ-change-151 (MODIFIED delta)
and the narrower invariant text, and names the journal when a legacy entry
points outside the project.
