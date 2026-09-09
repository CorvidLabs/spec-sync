---
id: insert-check-fix-export-rows-into-the-existing-public-api-table-and-state-the-strict-enforcement-default-in-cli-help
state: archived
type: bug_fix
base_commit: 38354c51ad3a4ba2408b8daf906dd1a9a0d5659f
---

# Insert check --fix export rows into the existing Public API table and state the strict enforcement default in CLI help

## Intent

Insert check --fix export rows into the existing Public API table and state the strict enforcement default in CLI help

## Affected Canonical Specs

- `cmd_check`
- `cli_args`

## Acceptance Criteria

- check --fix inserts a new export row directly after the last row of the nearest Public API table in the section or subsection, under a heading, a bold label, or no label, and preserves trailing prose; a section with no table still receives rows at its end; a subsequent check --strict passes on the fixed spec; the --enforcement help summary names strict as the default; targeted fix integration tests, clippy, fmt, and strict spec validation pass

## No-spec Rationale

Not applicable
