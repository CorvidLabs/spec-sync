# Lesson bundle — drop-interpolated-next-action-from-v1-verifying-assert-messages-so-codeql-cleartext-logging-is-not-a-required-check

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Drop interpolated next_action from v1 verifying assert messages so CodeQL cleartext-logging is not a required-check failure
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change_tests.rs
- **Acceptance**: workflow_v1_verifying_next_action_names_verify_accept_archive keeps its substring assertions but its panic messages do not interpolate next_action; cargo test that test passes; CodeQL rust/cleartext-logging alerts 72 and 73 close

## Evidence

- Verification commit: `b76a2f87e5387e2402840e9b41251ff41f1f793f`
- Base commit: `0bdfffc065f1cd624fe272073642eb1898f8b595`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

PR #774 is merge-blocked by the CodeQL required check: 2 high `rust/cleartext-logging` alerts (#72, #73) on `src/change_tests.rs` lines 16596 and 16600.

Those lines are panic messages on `workflow_v1_verifying_next_action_names_verify_accept_archive` that interpolate `{next}` (`summarize_change(...).next_action`). CodeQL taint-tracks that string as sensitive. The substring assertions themselves are fine.

Drop the interpolations. Do not change product code or the assertions.

## From the change's testing.md

# Testing

Fail: CodeQL `rust/cleartext-logging` on the two `{next}` interpolations in `workflow_v1_verifying_next_action_names_verify_accept_archive`.

Pass: `cargo test --bin specsync -- workflow_v1_verifying_next_action`. Panic messages must not contain `{next}`.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-100 | `workflow_v1_verifying_next_action_names_verify_accept_archive` still asserts verify/accept/archive and rejects check/finalize; panic messages no longer interpolate `{next}` |

## Where these lessons go

- `specs/change/context.md`
