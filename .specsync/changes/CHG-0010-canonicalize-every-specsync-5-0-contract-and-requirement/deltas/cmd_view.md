## ADDED

### REQUIREMENT REQ-cmd-view-001

The view command SHALL render a validated role-specific canonical spec view and fail clearly for unknown roles or specs.

Acceptance Criteria
- `cmd_view` discovers spec files under `config.specs_dir` via `find_spec_files`
- Each spec is rendered through `view::view_spec(path, role)`, which keeps only the sections allowed for the given role
- When `spec_filter` is provided, only the spec whose module name (file stem minus `.spec`) matches is rendered
- Rendered specs are separated by a `---` delimiter line
- A per-file render error is printed to stderr (`error:` prefix) and processing continues to the next spec
- Exits with code 1 when no spec files are found
