---
change: CHG-0086-return-src-commands-change-rs-to-its-sole-canonical-owner
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-change-052` | `specs/cmd_change/cmd_change.spec.md` is again the sole claimant of `src/commands/change.rs`; `specsync check` reports no duplicate spec ownership, where it previously failed with `Source file has duplicate spec ownership`. Suites: 2,185 unit and 333 integration. |

## Notes

CI caught this, not the local gate: duplicate ownership is a project-wide
property, and nothing in the change lifecycle looks across all specs for a
second claimant.
