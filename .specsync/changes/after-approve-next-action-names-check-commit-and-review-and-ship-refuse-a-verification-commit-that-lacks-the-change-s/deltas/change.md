---
module: change
---

## ADDED

### REQUIREMENT REQ-change-156

After definition approval, `change status` and `change show`, and the domain
`next_action` they print, SHALL name `specsync change check <id> --commit` as
the next step for an approved or implementing change whose approval is current
and whose artifacts are complete. They SHALL NOT name a bare
`specsync change check <id>` for that step, because following that command can
verify and then complete the lifecycle with the change's work still
uncommitted.

#### Acceptance Criteria

- After `change approve` of a change whose affected path is modified in the
  working tree and not committed, text and JSON `next_action` for
  `change status` and `change show` contain `specsync change check <id> --commit`
  and do not name a bare `specsync change check <id>` as the next command.
- The same wording applies to an implementing change in the same condition.
- Open tasks still appear first, as REQ-change-132 requires; the command that
  follows them is `specsync change check <id> --commit`.
- A verifying change that already names `change check` as its next step names
  `--commit` on that command.
- Bare `change check` without `--commit` remains a valid command
  (REQ-cli-args-010); this requirement only changes the guided next action.

### REQUIREMENT REQ-change-157

`change review` and `change ship` SHALL refuse when the recorded verification
commit does not contain the change's own work — the current contents of its
`affected_paths`, and any canonical specs the change materialized. The refusal
SHALL name that commit and `specsync change check <id> --commit`. They SHALL
NOT record a scoped review, and SHALL NOT finalize or archive, in that case.

A passing review SHALL bind only to a verification commit that contains that
work. Uncommitted files under `.specsync/` alone SHALL NOT trigger this
refusal: the review then finalize path leaves that evidence uncommitted by
design (REQ-change-093, change.spec invariant 42).

#### Acceptance Criteria

- On a no-spec documentation change whose affected path is modified and
  uncommitted, after a passing `change check` without `--commit`,
  `change review` exits non-zero, records no review, and does not bind a pass
  to the pre-change commit.
- `change ship` in the same state exits non-zero and does not finalize or
  archive the change.
- After `change check --commit` records verification against a commit that
  contains the affected-path work, `change review` and `change ship` may
  proceed as they do today.
- A change whose only uncommitted files are under `.specsync/` is not refused
  by this rule.

## MODIFIED

### REQUIREMENT REQ-change-132

`change status` SHALL name the open tasks and how to close them before the
command that would otherwise be next, instead of naming only that command.
This SHALL apply to an approved, implementing or verifying change whose
approval is current, whose artifacts are complete and whose `tasks.md` has an
open task. It SHALL apply in the text `Next:` line and in JSON
`summary.next_action`.

#### Acceptance Criteria

- With one open task on an approved change, text and JSON status name the task
  file, the task's line and text, and then `specsync change check <id> --commit`.
- After a passing check with one task reopened, text and JSON status name the
  open task before the review step.
