## ADDED

### REQUIREMENT REQ-change-audit-project-001
The change module SHALL expose `audit_project` that validates active change workspaces and living SDD policy/spec coherence without rewalking archived terminal evidence by default.

Acceptance Criteria
- `audit_project` does not load or re-authenticate every archived change's terminal evidence.
- `check_project` remains available for full integrity including archives (tests / rare callers).
- CLI project-health surface uses the active-only path.

### REQUIREMENT REQ-change-check-scoped-002
`check_change` SHALL continue to materialize approved deltas and run verification for one selected change only; project-wide archive integrity is not part of that function.

Acceptance Criteria
- Selecting zero, one, or many open changes behaves as before (nothing / that id / error listing ids).
- Archive terminal evidence is not required for a successful scoped check.
