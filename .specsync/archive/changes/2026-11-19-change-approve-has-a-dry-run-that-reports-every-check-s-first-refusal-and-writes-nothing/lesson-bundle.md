# Lesson bundle: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Change approve has a dry run that reports every check's first refusal and writes nothing
- **Kind**: Feature
- **Specs**: change, cmd_change, cli_args
- **Paths**: src/change.rs, src/change/, src/change_tests.rs, src/change_tests/, src/commands/change.rs, src/cli.rs, tests/integration.rs, tests/integration/, specs/change/, specs/cmd_change/, specs/cli_args/, site/src/content/docs/, CHANGELOG.md, AGENTS.md
- **Acceptance**: `specsync change approve <id> --dry-run` says whether the approval would be accepted, in text and with `--json`, and exits 0 when it would and 1 when it would not.
- **Acceptance**: It writes nothing: every file in the project, with ignored and untracked files, the Git index and SpecSync's caches, is byte for byte unchanged after it, for a refused draft and for a clean one.
- **Acceptance**: It runs `change approve`'s own checks in the same order through the same functions: for each of the 25 refused drafts on the Chough tester's list the dry run reports the message the real approve gives, and the real approve afterwards still refuses with it; a clean draft reports nothing and exits 0, and the real approve then succeeds.
- **Acceptance**: Each refusal is its own JSON entry carrying the check's name and the message `change approve` prints.
- **Acceptance**: It works on a draft and on a change that was sent back.
- **Acceptance**: It runs every check that can run on its own, reports the first problem from each, and names each check it skipped with the refused check it waits on.
- **Acceptance**: `change approve` without `--dry-run` refuses exactly what it refused before, with the same messages, and approves what it approved before.
- **Acceptance**: With `--json`, a refused `definition` or `ownership` check whose message names a `specsync change answer` command carries that command as `fix`, taken from the code that writes the message. A `fix` may hold a placeholder, and the docs say it must not be run blindly.

## Evidence

- Verification commit: `3fb40d8f235076b2acc233a9cbe2762f0eb06748`
- Base commit: `3bebe8b03094b2079630801584b56f94a1e460c5`
- Verified by: `specsync check --spec change --spec cli_args --spec cmd_change`

## From the change's context.md

# Context

Chough's `change ready` tries to say whether `specsync change approve` will accept a draft, and it keeps
its own copy of some of SpecSync's rules to do it. The copy lags. Against SpecSync 6.3.0 the Chough tester
found 25 kinds of draft where `change ready` says "Nothing is missing." and approve then refuses: a wrong
heading in a delta, an empty item or file, the same item twice, an ADDED requirement whose id exists or that
has no Acceptance Criteria, another module's requirement id, a path no module owns, a spec that does not
parse, and more (CorvidLabs/chough#311, #406). Issue #1091 asks SpecSync for the one source of truth: a dry
run of the approval that Chough can call.

Leif approved #1091 under the feature freeze on 2026-11-18, for the coming release. On depth he chose "First
problem from each check": run every check that can run on its own, report the first problem from each, and
say which checks were skipped. Listing every problem inside one check would mean reworking the shared
validators, and it is not in this release.

Leif approved this definition on 2026-11-19 at about 05:06Z in a Claude Code chat with his planning session
(hc t4c), relayed through the spec-sync lead agent (ss-lead, hc t58): "OK, with (a) fixed" for the intent
(CHANGE-15 to CHANGE-15.c) and "Approve" for the requirements, the criteria and four choices: which checks
are skipped, that the dry run takes no lock, that it refuses when an interrupted write is waiting, and that
the actor is its own check.

What a session picking this up needs to know:

- A refused `change approve` writes exactly one thing today: `.specsync/change.lock`, when it is absent.
  Taking the lock also recovers an interrupted lifecycle write, which is a write. So the dry run takes no
  lock and reads as `change status` does.
- The approval's checks live in `approve_definition_with_relay` (`src/change/sdd_version.rs`) and in the
  two functions that build the approval record (`src/change/validate_loaded_change.rs`). The dry run must
  walk the same list, not a copy of it.
- A real refusal prints `{"error": "<message>"}` with `--json` and `error: <message>` in text. The dry
  run prints its own document and never that one.
- REQ-change-163 and REQ-cmd-change-040 were free when this was written: 161 is taken by #1089, and 162, 164
  and 165 by branches from 2026-11-13; REQ-cmd-change-038 by #1089 and 039 by #1093.

## From the change's design.md

# Design

## One list of checks, walked two ways

A new file, `src/change/approval_checks.rs`, holds the approval's checks as one ordered list,
`ApprovalStep`, and one context, `ApprovalRun`, whose `check(step)` runs one step and writes nothing.
`approve_definition_with_relay` and the dry run both walk that list, so a check added to one is added to
the other.

| Step | Name in the report | What it runs | Waits on |
|---|---|---|---|
| Relay | `relay` | `relayed_decision`'s result, `--actor` present, `ensure_relay_names_the_decider` | nothing |
| LifecycleLock | `lifecycle_lock` | approve: `acquire_project_lock`. Dry run: `reject_symlink_components_for` on the lock path, and whether an interrupted write is waiting | nothing |
| Change | `change` | `load_change` | `lifecycle_lock` |
| Sequence | `sequence` | `ensure_no_sequence_collision` | `lifecycle_lock`, `change` |
| State | `state` | the state guard, with the accepted-refresh exception | `lifecycle_lock`, `change` |
| Roster | `roster` | `list_changes_checked` | `lifecycle_lock`, `change`, `state` |
| Baseline | `baseline` | `bind_legacy_archive_baseline_authority`, on the record in memory | the same |
| Definition | `definition` | `validate_definition` | the same |
| Deltas | `deltas` | `validate_delta_files` | the same |
| Ownership | `ownership` | `validate_declared_path_ownership` | the same |
| VerificationIgnoredPaths | `verification_ignored_paths` | `ensure_change_clear_of_verification_ignored` | the same |
| ApprovalRecord | `approval_record` | the definition digest, the approvals ledger, the approved scope, the delta body digests and bases; the portable pair for `--portable-5-0-1` | the same, and `deltas` |
| Actor | `actor` | `resolve_actor` | the same as `roster` |

The real approval walks the list and returns the first `Err`, exactly as it does today, then writes. Its
two writes keep their places: the workflow-version raise for a relayed decision is saved after
`verification_ignored_paths`, and the ledger, `state.json` and `change.md` after `actor`.

The dry run walks the same list. A step whose prerequisite was refused is reported `skipped` with that
prerequisite as `skipped_because`. Every other step runs, and its first `Err` becomes its `message`. The
baseline binding and the workflow-version raise stay on the record in memory.

Building the approval record is split from writing it. `append_approval_with_relay` and the portable pair
function each become a read-only part that builds what will be written, and a write. The actor is resolved
last, as it is today.

## The lock step

The dry run does not acquire the lifecycle lock, because acquiring it creates `.specsync/change.lock` and
recovers an interrupted write. It runs the lock's own no-symlink walk on the lock path, so that refusal is
predicted, and it reads whether `.specsync/change-transaction.json` exists. If it does, the step is
refused: a real approval would first recover that write, and the dry run does not guess what the tree looks
like after. Every later step is then skipped.

## What the caller gets

`change::approve_definition_dry_run` returns `ApprovalDryRun`: the change id, `dry_run: true`,
`would_approve`, and `checks`, one `ApprovalCheckReport` per step in order with `check`, `result`
(`passed`, `refused`, `skipped`), `message` and `skipped_because`. `would_approve` is true only
when no step is refused.

`specsync change approve <id> --dry-run` prints that value as one JSON document with `--json`. In text it
prints one line per refused step, one per skipped step, and a closing line. It exits 1 when
`would_approve` is false, after printing, without the `error` document.

## The `fix` field (after the send-back of 2026-11-19)

`validate_definition` and `validate_declared_path_ownership` gain `_with_fix` forms that return a
`RefusalWithFix`: the message, and the bare command it names. The command comes from
`answer_command` and `declare_owners_command`, the plain forms of `answer_command_hint` and
`declare_owners_answer`, which now build their text from them, so no message changes. The approval
reduces the refusal to its message; the dry run also reports the command as `fix`.

## Not changed

No validator changes what it accepts or what it says. `change approve` without `--dry-run` keeps its
order, its messages, its writes and its exit codes.

## From the change's testing.md

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

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
- `specs/cli_args/cli_args.3md`
