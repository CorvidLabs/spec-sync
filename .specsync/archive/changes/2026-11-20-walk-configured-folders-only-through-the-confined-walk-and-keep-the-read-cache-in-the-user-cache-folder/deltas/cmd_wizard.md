## ADDED

### REQUIREMENT REQ-cmd-wizard-004

The wizard SHALL find candidate source files only through the confined, budgeted walk of `util`, so a source folder that leads out of the project is never walked.

Acceptance Criteria
- The wizard's source walk uses `find_project_files`.
