## ADDED

### REQUIREMENT REQ-cmd-check-016

`check --fix` SHALL ignore fenced Markdown when locating `###` export subsections and near-miss headings. A fenced `### Exported Functions` is quoted text, not an insertion target.

Acceptance Criteria
- A Public API section whose first `### Exported Functions` is inside a fence, followed by a real heading and table, receives new rows in the real table.
- The fenced sample is preserved verbatim.
- Near-miss header rewrites also run on fence-blanked text so a fenced `### Exported Functons` is not renamed.
