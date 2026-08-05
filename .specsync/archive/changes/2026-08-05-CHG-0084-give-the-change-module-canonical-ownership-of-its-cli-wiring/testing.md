---
change: CHG-0084-give-the-change-module-canonical-ownership-of-its-cli-wiring
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-change-052` | The change spec `files:` list contains `src/commands/change.rs`; `cargo test` passes 2,181 unit and 333 integration tests; `change finalize` on a change whose acceptance inputs include that file resolves ownership instead of rejecting it. |

## Follow-up

Canonical ownership is enforced at finalize but not at check or approve, so this
class of failure surfaces only after verification has already succeeded. Moving
the check earlier is tracked separately.
