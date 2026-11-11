# Lesson bundle: journal-read-errors-name-the-file-and-a-quoted-recovery

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Journal read errors name the file and a quoted recovery
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/sdd_version.rs, src/change_tests/transaction_journal_error.rs
- **Acceptance**: a non-UTF-8 or unreadable journal gets the same path+recovery error as an unparseable one; the printed recovery works verbatim in a project whose path contains a space

## Evidence

- Verification commit: `bcfd9c3271db222cb1571730f676884efdb95a1b`
- Base commit: `2321597cf803f495c7491901b088a040fe42a457`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Round 2 of #433 (PR #1041), t384's review. Two gaps in round 1: the journal
READ branch (non-UTF-8 bytes, a directory at the path) still returned a bare
error, and the printed `mv` recovery was unquoted, so it failed verbatim in a
project whose path contains a space. Both are the same promise as round 1: a
refusal names the file and a recovery that runs as printed.

Also adopts the reviewer's smaller wording for REQ-change-151 (MODIFIED delta)
and the narrower invariant text, and names the journal when a legacy entry
points outside the project.

## From the change's testing.md

# Testing

change_tests/transaction_journal_error.rs gains non-UTF-8, directory and
spaced-root cases.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-151 | change_tests journal tests incl. spaced-root recovery |

## Where these lessons go

- `specs/change/change.3md`
