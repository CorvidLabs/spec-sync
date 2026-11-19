---
id: cover-the-call-sites-that-pass-the-project-root-to-change-3-s-reads
state: approved
type: bug_fix
base_commit: 6d524c6c2f5c4579b81bd4acca85651834be45da
---

# Cover the call sites that pass the project root to change 3's reads

## Intent

Cover the call sites that pass the project root to change 3's reads

## Affected Canonical Specs

- `cmd_init_registry`
- `cmd_issues`
- `cmd_score`
- `cmd_view`
- `mcp`

## Acceptance Criteria

- specsync change audit reports no meaningful changed path that no change covers, with the six call-site files declared here; the files are unchanged by this change; the full test suite still passes

## No-spec Rationale

Each of the six files only passes the new project root argument to a function change 3 gave one (validate_config_file, schema_read_errors, filter_by_status, view_spec, read_config_file, build_schema). The code was committed and reviewed with change 3; no contract changes.
