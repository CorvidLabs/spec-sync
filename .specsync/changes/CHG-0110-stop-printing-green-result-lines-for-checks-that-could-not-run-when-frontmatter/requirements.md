---
change: CHG-0110-stop-printing-green-result-lines-for-checks-that-could-not-run-when-frontmatter
artifact: requirements
---

# Requirements

## REQ-commands-008

Validation output SHALL NOT report a successful result for a check that could not run.

Acceptance Criteria
- When frontmatter is invalid, the source-file, required-section, and dependency checks are each reported as skipped rather than as passing.
- The skipped form matches the existing vocabulary used when a draft spec skips validation.
- A spec with valid frontmatter continues to report all three checks unchanged.
- The exit status is unaffected: invalid frontmatter remains an error.
