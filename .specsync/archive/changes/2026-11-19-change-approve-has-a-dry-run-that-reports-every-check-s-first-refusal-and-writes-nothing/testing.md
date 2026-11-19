---
change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
artifact: testing
---

# Testing

Failing first: the tests below are committed before the code, and fail there because `--dry-run` is not
an argument `change approve` takes.

- `tests/integration/approve_dry_run.rs`
  - A table of complete drafts with one fault each, from the Chough tester's list of 25 plus the unfinished
    interview. For each row: the dry run names the expected check and message and exits 1, every file of
    the project is unchanged, and the real `change approve` then refuses with that same message.
  - A clean draft: no refusal, exit 0, an unchanged tree, and the real approve then succeeds.
  - The unchanged tree is compared with and without an existing `.specsync/change.lock` and
    `.specsync/hashes.json`. It covers ignored and untracked files, the Git index and the read cache.
  - An open interview question and an invalid delta in one draft: two refusals, each under its own check.
  - A change id that does not exist: one refusal, every later check skipped.
  - A change that was sent back: the dry run says what its re-approval would.
  - `--words` with no `--where`: the `relay` check is refused with the approval's message.
  - `--json` prints one document, refused or clean, and never the `error` document.
- `src/change_tests/approval_dry_run.rs`: the three skip rules, the baseline binding staying in memory,
  and an interrupted write refusing with every later check skipped.
- `src/cli.rs` tests: `change approve <id> --dry-run` parses, with and without the other flags.
- The existing approve tests in `src/change_tests/` and `tests/integration/` pass unchanged.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-change-163` | `tests/integration/approve_dry_run.rs`: the refused-draft table, the clean draft, the unchanged tree, two problems in one run, the missing id and the sent-back change; `src/change_tests/approval_dry_run.rs`: the skip rules, the in-memory binding and the interrupted write |
| `REQ-cmd-change-040` | `tests/integration/approve_dry_run.rs`: the `fix` field (a definition refusal's command with its placeholder, an ownership refusal's command run as printed, no `fix` elsewhere), one text line per refused check, text lines and exit codes, JSON as one document, the first refused message equal to the real approve's `error`, the relay flags |
| `REQ-cli-args-027` | the `change approve --dry-run` parsing test in `src/cli.rs` |
