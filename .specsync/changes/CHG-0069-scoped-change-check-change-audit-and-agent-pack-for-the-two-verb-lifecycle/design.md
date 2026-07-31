---
change: CHG-0069-scoped-change-check-change-audit-and-agent-pack-for-the-two-verb-lifecycle
artifact: design
---

# Design

## Two-verb model

| Verb | Scope | Cost model |
|------|--------|------------|
| `change check` | One change verify | O(verification commands) |
| `change audit` | Active + living specs | O(active changes) |

## Implementation seam

`check_project_with_command_output(..., include_archive_integrity: bool)`

- `true` → legacy full integrity for tests (`check_project`)
- `false` → `audit_project` / quiet comment path

CLI `Check` never calls full integrity.
