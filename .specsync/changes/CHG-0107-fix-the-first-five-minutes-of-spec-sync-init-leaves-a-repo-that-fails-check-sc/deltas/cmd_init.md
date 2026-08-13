## ADDED

### REQUIREMENT REQ-cmd-init-005

Initialization SHALL leave a repository that passes the very next lifecycle check.

Acceptance Criteria
- The protected SDD paths initialization creates are recorded in `.specsync/bootstrap.json`.
- The first check after initialization reports no uncovered meaningful delivery for files
  initialization itself wrote.
- Failure to write the record is reported as a warning and does not fail initialization.
