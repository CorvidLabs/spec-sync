---
change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
artifact: tasks
---

# Tasks

- [x] Failing integration tests: the refused-draft table with the real approve after each row, the clean draft, the unchanged tree, two problems in one run, a missing id, a sent-back change, the relay flags, JSON as one document
- [x] Failing argument test: `change approve` accepts `--dry-run`
- [x] One ordered list of approval steps in `src/change/approval_checks.rs`, walked by `approve_definition_with_relay`
- [x] Building the approval record split from writing it, for the plain and the portable approval
- [x] `approve_definition_dry_run`: first problem per step, the three skip rules, no lock, no write
- [x] `--dry-run` on `change approve`: text lines, one JSON document, exit 0 or 1
- [x] Unit tests for the skip rules, the in-memory baseline binding and the interrupted write
- [x] Public API rows and `files:` entries in the `change`, `cmd_change` and `cli_args` specs
- [x] Docs: `cli.md`, `CHANGELOG.md`, the `AGENTS.md` command table, a lesson plane in `change.3md`
- [x] `fledge lanes run verify` and the pre-push gate
- [x] After the send-back of 2026-11-19: failing tests for the `fix` field (definition and ownership, the placeholder, run as printed, none elsewhere), a lock path approve cannot open, and one text line per refused check
- [x] The `fix` field from `answer_command` and `declare_owners_command`, the bare forms of the helpers that write the message; the lock path opened as approve opens it; text output joins a message's lines
- [x] Docs for the `fix` field and the lock check: `cli.md`, `CHANGELOG.md`, the `ApprovalCheckReport` row
- [x] `fledge lanes run verify` and the pre-push gate on the re-scoped change

Human gates and Chough's own records are not tasks here: Leif's scope approval and implementation review
are recorded through Chough when he gives them.
