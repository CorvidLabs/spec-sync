---
change: CHG-0091-add-change-ship-status-for-local-ship-readiness-and-merge-before-finalize-warning
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-cli-args-011` | `change --help` / clap parse includes ShipStatus; optional ID. |
| `REQ-cmd-change-007` | Manual: verifying change with valid verification reports tip health; orphaned commit produces blocker; ready_to_finalize only when review present and tip ok. |

## Tasks

- [x] Implement ship-status command
- [x] Merge-before-finalize warning on verifying next_action
