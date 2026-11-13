---
change: doctor-fix-refuses-under-an-inherited-verification-context-and-read-only-doctor-still-reports
artifact: requirements
---

# Requirements

REQ-cli-003 is modified (`deltas/cli.md`): the dispatcher also fails closed for `doctor --fix`,
with or without `--dry-run` or `--close-late`, under an inherited verification context, and a
`doctor` without `--fix` still reports there with the same output and exit status. The cli
spec's Purpose names `doctor --fix` among the refused commands. No other requirement changes.
