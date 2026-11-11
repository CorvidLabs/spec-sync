# Lesson bundle: scaffold-prints-the-empty-files-warning-when-every-candidate-source-is-owned-by-another-spec

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Scaffold prints the empty-files warning when every candidate source is owned by another spec
- **Kind**: BugFix
- **Specs**: cmd_scaffold
- **Paths**: src/commands/scaffold.rs, tests/integration/config.rs
- **Acceptance**: scaffold foo, where every candidate source file is already listed by another spec, writes files: [] and prints the same empty-files warning on stderr that new prints; the scaffold, single-source-fallback and new ownership integration tests assert files: [] and that warning

## Evidence

- Verification commit: `bc0302a9f95df609b6807a2268ec3b7e13194133`
- Base commit: `1fffafcbcf05e179b0d92139d709a6bf146abcb8`
- Verified by: `specsync check --spec cmd_scaffold`

## From the change's context.md

# Context

PR #1025 (Fixes #611), round 3. The corvid-agent review at `53c1fa5e` found that
REQ-cmd-scaffold-006 AC2, reworded in round 2, says an all-owned candidate set takes
the no-files path: `files: []` plus the empty-files warning. `specsync new foo`
prints that warning (`cmd_add_spec` in `src/commands/scaffold.rs`); `specsync
scaffold foo` wrote `files: []` and printed nothing on stderr.

The requirement already says what should happen, so no canonical spec changes: both
commands now call one shared warning helper, and the ownership integration tests
assert `files: []` and the warning on stderr.

Authorization: Leif's 2026-11-10 merge rule lets bug fixes that make behavior match
existing specs run the full lifecycle with 0xLeif as approve and review actor.

## From the change's testing.md

# Testing

Integration tests in `tests/integration/config.rs` run the built binary against a
fixture where another spec already lists the only candidate source file.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-cmd-scaffold-006 | integration `scaffold_never_claims_a_file_another_spec_owns`, `scaffold_single_source_fallback_skips_an_owned_file` (stderr warning and `files: []`) |
| REQ-generator-009 | integration `new_never_claims_an_owned_flat_file` (stderr warning and `files: []`) |

## Where these lessons go

- `specs/cmd_scaffold/cmd_scaffold.3md`
