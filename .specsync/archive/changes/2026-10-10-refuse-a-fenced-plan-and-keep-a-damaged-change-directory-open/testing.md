---
change: refuse-a-fenced-plan-and-keep-a-damaged-change-directory-open
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-050 | `active_directory_with_files_and_no_state_is_still_open` passes, and an empty deltas husk is still skipped. |
| REQ-change-068 | The same test reports the missing state file and keeps sibling enumeration. |
| REQ-change-105 | `a_fenced_plan_cannot_be_completed_from_inside_a_code_block` passes, and the existing plan-section tests still pass. |
| REQ-cmd-change-019 | `.github/scripts/test-archive-readiness.sh` names `broken-notes` as still open and exits 0 for an empty husk. |

- `cargo test --bin specsync -- fenced_plan` passed.
- `cargo test --bin specsync -- active_directory_with_files_and_no_state_is_still_open` passed.
- `bash .github/scripts/test-archive-readiness.sh` passed.
