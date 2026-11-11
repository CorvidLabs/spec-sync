# Lesson bundle: journal-error-names-its-file-and-recovery

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Journal error names its file and recovery
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/sdd_version.rs, src/change/validate_loaded_change.rs, src/change/consider_accepted_evidence_anchor.rs, src/change_tests.rs, src/change_tests/, tests/
- **Acceptance**: a corrupt change-transaction.json error names the path and how to set it aside; every printed 'change reopen' remediation carries --actor and --reason

## Evidence

- Verification commit: `71e4afbb04ec9bcefe299e395c921637da672e67`
- Base commit: `46f55652649a72ddf5e1715938b504b96686566b`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Issue #433 (part 3 + remediation strings). A corrupt or leftover
`.specsync/change-transaction.json` bricks every write with a raw serde error
(`invalid transaction journal: expected value at line 1 column 1`) — no file
path, no recovery hint. And the stale-evidence refusals print
`specsync change reopen <id>` with no `--actor`/`--reason`, so the pasted
command fails verbatim (`doctor` already prints the complete form).

The journal records an interrupted multi-file write; a corrupt one cannot be
replayed. The recovery is to inspect it and set it aside, after checking no
write was in flight. The fix names the path and that recovery in the error,
and appends the required flags to every reopen remediation.

## From the change's testing.md

# Testing

Unit/integration: corrupt journal error names the path; reopen remediations
carry both flags (update the existing assertions that pin the bare form).

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-151 | change_tests + integration assertions on both message families |

## Where these lessons go

- `specs/change/change.3md`
