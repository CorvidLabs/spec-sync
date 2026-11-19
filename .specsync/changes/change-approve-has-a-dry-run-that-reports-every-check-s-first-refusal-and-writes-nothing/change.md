---
id: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
state: implementing
type: feature
base_commit: 3bebe8b03094b2079630801584b56f94a1e460c5
---

# Change approve has a dry run that reports every check's first refusal and writes nothing

## Intent

change approve has a dry run that reports every check's first refusal and writes nothing

## Affected Canonical Specs

- `change`
- `cmd_change`
- `cli_args`

## Acceptance Criteria

- `specsync change approve <id> --dry-run` says whether the approval would be accepted, in text and with `--json`, and exits 0 when it would and 1 when it would not.
- It writes nothing: every file in the project, with ignored and untracked files, the Git index and SpecSync's caches, is byte for byte unchanged after it, for a refused draft and for a clean one.
- It runs `change approve`'s own checks in the same order through the same functions: for each of the 25 refused drafts on the Chough tester's list the dry run reports the message the real approve gives, and the real approve afterwards still refuses with it; a clean draft reports nothing and exits 0, and the real approve then succeeds.
- Each refusal is its own JSON entry carrying the check's name and the message `change approve` prints.
- It works on a draft and on a change that was sent back.
- It runs every check that can run on its own, reports the first problem from each, and names each check it skipped with the refused check it waits on.
- `change approve` without `--dry-run` refuses exactly what it refused before, with the same messages, and approves what it approved before.

## No-spec Rationale

Not applicable
