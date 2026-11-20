## ADDED

### REQUIREMENT REQ-commands-016

`check_case_collision` SHALL list the specs folder only through `read_project_dir` (REQ-util-010); a specs folder that leads out of the project is not listed, so nothing outside decides a module name.

Acceptance Criteria
- A committed `specs` link out holding `Auth` does not refuse `auth` as a collision.
