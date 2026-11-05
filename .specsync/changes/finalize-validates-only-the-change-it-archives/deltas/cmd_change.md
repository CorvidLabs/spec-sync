## ADDED

### REQUIREMENT REQ-cmd-change-027

`specsync change audit --archives` SHALL also report every archived change whose own workflow-version history fails validation, through the domain's `audit_project_with_archives`, and exit non-zero when it finds one.

#### Acceptance Criteria

- `change audit --archives` exits non-zero and names the archived change when another archive's history holds a hidden workflow-version downgrade.
- `change audit` without `--archives`, with or without the global `--strict`, reports exactly what it reported before.
