## ADDED

### REQUIREMENT REQ-cmd-change-026

`specsync change audit` under the global `--strict` flag SHALL also report every archived change whose own workflow-version history fails validation, through the domain's `audit_project_with_strict`, and exit non-zero when it finds one.

#### Acceptance Criteria

- `change audit --strict` exits non-zero and names the archived change when another archive's history holds a hidden workflow-version downgrade.
- `change audit` without `--strict` reports exactly what it reported before.
