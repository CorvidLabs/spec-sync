# Lesson bundle: check-commit-json-prints-the-outcome

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Check commit json prints the outcome
- **Kind**: BugFix
- **Specs**: cmd_change
- **Paths**: src/commands/change.rs, tests/integration/change.rs
- **Acceptance**: check --commit --json prints one JSON outcome document carrying change, both commit ids, pushed, left_out, and configured_commands_not_run; a refusal prints the JSON error document with a non-zero exit

## Evidence

- Verification commit: `40654d17ea1dd7f3a43db6369ffcc5dd51c4f175`
- Base commit: `d93cad4615c3b1de5dfd0b2fa19425f06a2bbad6`
- Verified by: `specsync check --spec cmd_change`

## From the change's context.md

# Context

Issue #839, re-cut per t370/t384: the behavior is required by REQ-change-004
and cli invariant 5, but REQ-cmd-change-unrun-commands-001's third acceptance
line pinned the empty-output behavior ("gains no success document"), so the
spec was self-inconsistent. t384's option (a): the outcome document carries
`configured_commands_not_run` like every other JSON success output, and the
AC/item-13 exemption is retired to match. READY track (retiring contract text
is Leif's call at merge).

## From the change's testing.md

# Testing

tests/integration/change.rs: change_check_commit_json_prints_the_outcome
asserts the outcome document including configured_commands_not_run.

tests/integration/change.rs: change_check_commit_json_refusal_prints_one_error_document
covers the refusal criterion: with one task open, `change check <id> --commit --json`
exits non-zero, prints exactly one JSON document with an `error` key on stdout and
nothing else, and commits nothing. The refusal path already behaved this way, so there
is no red run against main; mutations show it discriminates: `say` printing progress
to stdout in JSON mode, and the error handler printing a second document, each fail it.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-cmd-change-unrun-commands-001 | the extended integration test |
| REQ-change-004 | both integration tests (success outcome and refusal) |

## Where these lessons go

- `specs/cmd_change/cmd_change.3md`
