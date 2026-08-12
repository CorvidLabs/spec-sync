## ADDED

### REQUIREMENT REQ-cmd-comment-004

The comment command SHALL report spec-check results only and SHALL NOT fold SDD lifecycle
findings into them.

Acceptance Criteria

- Lifecycle errors and warnings are absent from the reported error and warning totals.
- Lifecycle state does not contribute to the command's exit status.
- Spec validation findings, coverage, and the bounded markdown protocol are unchanged.
