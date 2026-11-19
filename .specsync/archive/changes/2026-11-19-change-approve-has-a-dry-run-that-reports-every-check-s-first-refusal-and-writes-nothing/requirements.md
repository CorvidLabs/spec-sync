---
change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
artifact: requirements
---

# Requirements

Three requirements are added, one per affected module. Their text is in the deltas and is the text Leif
approved, with the one exception he added to REQ-change-163.

- ADDED REQ-change-163 (`deltas/change.md`): the dry run itself. It runs the approval's own checks in the
  same order through the same functions, reports the first problem from each check that can run, names the
  checks it skipped, and creates, changes or removes no file.
- ADDED REQ-cmd-change-040 (`deltas/cmd_change.md`): what `specsync change approve <id> --dry-run` prints
  in text and with `--json`, and its exit code, 0 or 1.
- ADDED REQ-cli-args-027 (`deltas/cli_args.md`): `change approve` accepts `--dry-run`.

Intent: CHANGE-15, CHANGE-15.a, CHANGE-15.b and CHANGE-15.c (`hi/change.md`).

No existing requirement changes. REQ-change-133 keeps its refusal texts, and the dry run reports them as
they are.
