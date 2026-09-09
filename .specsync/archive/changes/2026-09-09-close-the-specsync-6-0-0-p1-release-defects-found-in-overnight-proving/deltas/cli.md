## ADDED

### REQUIREMENT REQ-cli-011

The default `check` dispatch SHALL merge `--spec` flags with positional SPEC filters before calling `cmd_check`.

Acceptance Criteria
- A default (no-subcommand) invocation still runs Check with an empty spec filter.
- `specsync check --spec auth` and `specsync check auth` produce the same scoped validation.
