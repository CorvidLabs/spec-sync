## ADDED

### REQUIREMENT REQ-commands-008

Validation output SHALL NOT report a successful result for a check that could not run.

Acceptance Criteria
- When frontmatter is invalid, the source-file, DB-table, required-section, and dependency checks are each reported as skipped rather than as passing.
- The skipped form matches the existing vocabulary used when a draft spec skips validation.
- A spec with valid frontmatter continues to report all four checks unchanged, including a declared table absent from the schema and every genuinely missing required section.
- The exit status is unaffected: invalid frontmatter remains an error.
