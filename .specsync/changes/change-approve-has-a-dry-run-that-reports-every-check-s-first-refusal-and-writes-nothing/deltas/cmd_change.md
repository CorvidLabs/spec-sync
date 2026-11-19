---
change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
artifact: delta
module: cmd_change
---

## ADDED

### REQUIREMENT REQ-cmd-change-040

`specsync change approve <id> --dry-run` SHALL print the dry run of REQ-change-163 and record nothing. It SHALL take the flags `change approve` takes and check them as the approval does. In text it SHALL print one line for each refused check, with the check's name and its message, one line for each skipped check naming the refused check it waits on, and a closing line that says whether the change would be approved and that nothing was written. With `--json` it SHALL print one document holding the change id, `dry_run: true`, `would_approve`, and `checks`: one entry per check in approval order, with its `check` name, its `result` (`passed`, `refused` or `skipped`), the `message` of a refused check and the `skipped_because` check of a skipped one. It SHALL exit 0 when no check refuses and 1 when any does, and SHALL NOT print a refusal as the `error` document a real refusal prints.

Acceptance Criteria
- For a refused draft, the text names the check and its message and the command exits 1; with `--json`, stdout is one document with one `refused` entry carrying that message, and the exit code is 1.
- For a clean draft, `would_approve` is true, every check is `passed`, the exit code is 0, and the text says the change would be approved and that nothing was written.
- The first `refused` entry's `message` equals the `error` a real `change approve` with the same flags then prints.
- With `--words` and no `--where`, the `relay` check is refused with the approval's own message.
