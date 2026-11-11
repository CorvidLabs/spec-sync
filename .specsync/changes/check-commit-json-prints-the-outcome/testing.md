---
change: check-commit-json-prints-the-outcome
artifact: testing
---

# Testing

tests/integration/change.rs: change_check_commit_json_prints_the_outcome
asserts the outcome document including configured_commands_not_run.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-cmd-change-unrun-commands-001 | the extended integration test |
| REQ-change-004 | same |
