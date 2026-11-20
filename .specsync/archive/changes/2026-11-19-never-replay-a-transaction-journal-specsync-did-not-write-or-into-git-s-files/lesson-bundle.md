# Lesson bundle: never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Never replay a transaction journal SpecSync did not write, or into Git's files
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/, src/change.rs, src/change_tests/, src/change_tests.rs, tests/integration.rs, tests/integration/, specs/change/, hi/setup.md, INTENT.md, CHANGELOG.md
- **Acceptance**: A journal at .specsync/change-transaction.json that Git tracks, in the index or in HEAD, is never replayed: the writing change command refuses and names the git rm fix. A journal entry with a .git component in any letter case, or with trailing dots or spaces, or naming a file outside .specsync/, the configured specs_dir and the registry-mapped spec files with their requirements.md and history fragments, refuses the whole journal and replays nothing. A bare-array journal is refused with the invalid-journal recovery. The person's own interrupted write is still recovered. The new tests fail on cf528407 and pass with the fix

## Evidence

- Verification commit: `be0b03bb8c1ff93afd8670c537ef08f0b79d61a2`
- Base commit: `cf5284076db2b692d8e8df8953eb6525d5952690`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

An independent review of the private advisory branch found F1. Every writing `change` command takes the project lock, and the lock replays `.specsync/change-transaction.json` before the command does anything else. The journal is a file in the working tree, so a repository can commit one. Its entries only passed `safe_project_path`, which stops `..` and links out but not `.git/config`, so a committed journal could set `core.fsmonitor` (a command at the next `git status`), replace a hook keeping its mode, or delete `.git/HEAD`. The bare-array form has no envelope, and the newer digest is unkeyed. This affects 5.0.0, where the journal and its replay arrived (#335), through 6.3.0.

Human decisions, relayed by the planning session:
- "yes" (2026-11-19, about 15:49Z) to three points: never replay a journal Git tracks; replay only to paths the lifecycle itself writes and refuse any `.git` component case-insensitively; drop the legacy form.
- "yes" (about 15:55Z) to the scope details: SETUP-8 as written; a tracked journal refuses the command with the `git rm` fix; "the spec tree" means `.specsync/`, `specs_dir` and registry-mapped specs with their `requirements.md` and `history/`; `.git` refused case-insensitively with trailing dots and spaces stripped; one entry outside refuses the whole journal; the legacy form refused with the existing message and recovery.
- Left out by the same decision: moving the journal into the Git directory, and keying the digest. With the two path rules, a forged journal can only write files the repository could commit directly.

The fix ships in 6.4.0 with the dry run (#1100).

## From the change's design.md

# Design

All of it is in `src/change/sdd_version.rs`, with the path rule beside `transaction_relative_path` in `collect_requirement_ids.rs`.

- **`journal_is_tracked(root)`:**
  - `git ls-files --error-unmatch` or `git cat-file -e HEAD:./…` succeeding means tracked.
  - Checked before the journal is parsed. When it is tracked, the replay refuses with its own message: `the transaction journal at … is tracked by Git, so it came with the repository and is never replayed; remove it with `git rm --cached .specsync/change-transaction.json` and delete the file`.
- **`replayable_transaction_path(root, relative)`:**
  - Refuses a component that equals `.git` after trailing dots and spaces are removed and ASCII is lowercased.
  - Judges the path after backslashes become separators, because the write (`normalize_project_path`) does the same.
  - Accepts a path under `.specsync/`, and a spec file, `requirements.md` or `history/*.md` under the configured `specs_dir`. A `specs_dir` of `.` therefore opens nothing else.
  - Accepts a path that equals a registry-mapped spec, is the `requirements.md` beside it, or is in its `history/` directory.
  - Refuses anything else with `entry \`<path>\` is outside what the change workflow writes`. A `.git` refusal names Git's own files.
- **Replay.**
  - The journal is parsed as JSON first. An array is the 5.x form and is refused with `a bare-array journal from SpecSync 5.x is no longer replayed`.
  - The durable form is checked as before.
  - Every entry is then checked, by `safe_project_path`, the rule above and the existing symlink-component check, before any entry is applied.
  - The first refusal returns the invalid-journal error, so nothing is replayed and the journal stays for the person to inspect.
- **Writing.** `validate_prepared_transaction_target` applies the same rule, so this build never journals a write it would refuse to replay.
- **The legacy enum arm is removed.**

## From the change's testing.md

# Testing

Fixtures are temporary directories with their own `git init`. Every journal targets files inside the fixture only.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-162 | `src/change_tests/journal_replay_targets.rs`: `a_journal_never_rewrites_or_deletes_git_files`, `a_backslash_entry_cannot_reach_git_files`, `specs_dir_dot_does_not_widen_replay`, `a_journal_git_tracks_is_never_replayed`, `one_entry_outside_the_change_workflow_refuses_the_whole_journal`, `a_bare_array_journal_is_no_longer_replayed`, `a_real_interrupted_write_is_still_recovered`, `a_registry_mapped_spec_is_recovered_and_its_neighbours_are_not`; `tests/integration/project_reads.rs`: `change_new_never_replays_a_committed_journal_into_git` |

Red first on `cf528407`: 5 of the 6 unit tests and the CLI test failed. The replay accepted every planted journal, and `change new` rewrote the fixture's `.git/config`. The control `a_real_interrupted_write_is_still_recovered` passed there, as it should. Three older tests were updated to the new rules: two now write the durable form, and one journals a change record instead of a file at the project root.

Amended before the review: t4c's read of `94e4551d` showed that the rule checked the raw entry while the write turns backslashes into separators, and that `specs_dir = "."` admitted every path. `a_backslash_entry_cannot_reach_git_files` and `specs_dir_dot_does_not_widen_replay` failed on `75dddd76` and pass with the amendment.

Then the full verify lane and the Linux container run.

## Where these lessons go

- `specs/change/change.3md`
