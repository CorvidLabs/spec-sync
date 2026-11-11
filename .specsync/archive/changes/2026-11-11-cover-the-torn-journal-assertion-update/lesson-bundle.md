# Lesson bundle: cover-the-torn-journal-assertion-update

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Cover the torn-journal assertion update
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change_tests/semantic_application_resolves_registry_backed_canonical_paths.rs
- **Acceptance**: the assertion edit is covered by a declared affected path

## Evidence

- Verification commit: `ef79eda1672a1dcb15daa520ab3e58435ee40c26`
- Base commit: `f7d234568795a17862a371c2ab1107cb9d5e16e1`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

t384 found the torn_canonical assertion update rode round 2 undeclared. This change declares the path; the edit itself was forced by the message change and is already committed.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-151 | the covered assertion passes in the full suite |

## How the covered assertion runs

`change check` executes only the in-process spec↔code sync (recorded as
`specsync check --spec change`); sdd.json verification commands are not
executed by the lifecycle. The assertion this change owns runs as
`cargo test torn_canonical_transaction_journal_fails_closed` — locally at the
final head and in hosted CI's `test` job.

## Where these lessons go

- `specs/change/change.3md`
