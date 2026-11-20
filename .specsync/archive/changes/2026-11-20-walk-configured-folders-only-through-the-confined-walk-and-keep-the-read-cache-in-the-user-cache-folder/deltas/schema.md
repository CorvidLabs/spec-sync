## ADDED

### REQUIREMENT REQ-schema-003

The schema snapshot SHALL list the schema folder only through `read_project_dir` (REQ-util-010), so a schema folder that leads out of the project is refused before any file in it is named.

Acceptance Criteria
- A committed `db` link out names no file outside.
