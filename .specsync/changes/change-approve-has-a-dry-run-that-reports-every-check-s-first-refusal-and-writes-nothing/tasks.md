---
change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
artifact: tasks
---

# Tasks

- [ ] Failing integration tests: the refused-draft table with the real approve after each row, the clean draft, the unchanged tree, two problems in one run, a missing id, a sent-back change, the relay flags, JSON as one document
- [ ] Failing argument test: `change approve` accepts `--dry-run`
- [ ] One ordered list of approval steps in `src/change/approval_checks.rs`, walked by `approve_definition_with_relay`
- [ ] Building the approval record split from writing it, for the plain and the portable approval
- [ ] `approve_definition_dry_run`: first problem per step, the three skip rules, no lock, no write
- [ ] `--dry-run` on `change approve`: text lines, one JSON document, exit 0 or 1
- [ ] Unit tests for the skip rules, the in-memory baseline binding and the interrupted write
- [ ] Public API rows and `files:` entries in the `change`, `cmd_change` and `cli_args` specs
- [ ] Docs: `cli.md`, `CHANGELOG.md`, the `AGENTS.md` command table, a lesson plane in `change.3md`
- [ ] `fledge lanes run verify` and the pre-push gate

Human gates and Chough's own records are not tasks here: Leif's scope approval and implementation review
are recorded through Chough when he gives them.
