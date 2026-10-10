---
change: close-a-fence-only-on-whitespace-and-count-a-symlink-as-a-file
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-050 | `active_directory_with_files_and_no_state_is_still_open` reports a symlink-only directory as unreadable. |
| REQ-change-068 | The same test keeps the healthy sibling and names the missing state file. |
| REQ-change-105 | `a_fenced_plan_cannot_be_completed_from_inside_a_code_block` refuses an info-string closer and accepts a bare longer closer. |
| REQ-cmd-change-019 | `.github/scripts/test-archive-readiness.sh` names `linked-notes` as still open. |

- `cargo test --bin specsync -- fenced_plan` passed.
- `cargo test --bin specsync -- active_directory_with_files_and_no_state_is_still_open` passed.
- `bash .github/scripts/test-archive-readiness.sh` passed.
