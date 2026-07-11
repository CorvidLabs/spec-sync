## ADDED

### REQUIREMENT REQ-cmd-comment-002
Generated pull-request comments SHALL report SDD lifecycle failures even when a project has no canonical spec files.

Acceptance Criteria
- Empty canonical discovery does not bypass SDD checking in comment mode.
- SDD-only errors render a failing comment with actionable detail.
