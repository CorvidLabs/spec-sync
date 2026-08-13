## ADDED

### REQUIREMENT REQ-validator-010

A `files:` entry that resolves to a directory inside the project root SHALL be a reported error and
SHALL NOT pass validation.

Acceptance Criteria

- A confined directory mapping produces a validation error rather than extracting zero exports and
  passing the Public API comparison vacuously.
- The finding is not reported as resolving outside the project root; escape diagnostics keep
  precedence over the directory branch.
- The accompanying fix names the source files beneath the directory using the same expansion module
  generation applies, excludes configured exclude directories, and is truncated with a remainder
  count beyond a fixed limit.
- Snapshot-based validation reports the same error without enumerating the ambient filesystem.
