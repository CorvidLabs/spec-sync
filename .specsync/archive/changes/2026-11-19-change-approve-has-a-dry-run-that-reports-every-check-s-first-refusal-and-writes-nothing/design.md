---
change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
artifact: design
---

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
