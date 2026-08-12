## ADDED

### REQUIREMENT REQ-change-058

The lifecycle check SHALL expose exactly one configured-command output behavior, and the
quiet-output variant used solely to keep lifecycle findings out of a machine-consumed
report stream SHALL NOT exist.

Acceptance Criteria

- No lifecycle entry point suppresses configured verification command output; every
  invocation inherits the parent streams.
- The quiet-output check path and its selector type are absent rather than retained
  unused, so no caller can reintroduce the suppressed-output behavior.
- Verification command execution, failure reporting, and recursion refusal are otherwise
  unchanged.
