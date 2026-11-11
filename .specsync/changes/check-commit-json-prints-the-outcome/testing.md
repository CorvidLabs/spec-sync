---
change: check-commit-json-prints-the-outcome
artifact: testing
---

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
