## ADDED

### REQUIREMENT REQ-validator-053

Requirement ID checks and `find_files_named` SHALL walk `specs_dir` only through the confined, budgeted walk, and a missing file's suggestion SHALL list its folder only through `read_project_dir` (REQ-util-010), so a folder that leads out of the project is refused, never walked or listed.

Acceptance Criteria
- A `specs` link out yields a refusal and no requirement IDs from outside; a suggestion for `../outside/...` is none.
