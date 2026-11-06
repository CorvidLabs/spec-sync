## ADDED

### REQUIREMENT REQ-cmd-change-031

`change ship`, including `--dry-run`, SHALL refuse a change with an open task before finalizing anything. The refusal SHALL name each open task and `specsync change ship <id>` with the flags it was given. `change ship-status` SHALL report the open tasks as a blocker and SHALL NOT report the change ready to finalize. Its `Next:` SHALL name the open tasks before the ship command.

#### Acceptance Criteria

- After a passing check and review, reopening one task makes `change ship-status <id>` list an incomplete-tasks blocker with `ready_to_finalize` false. It also makes `change ship <id>` exit 1 without archiving, naming the task and `specsync change ship <id>`.
- Ticking the box and running the named `change ship` command archives the change.
