---
change: parse-a-broken-policy-beside-a-baseline-and-retry-the-archive-commit
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-108 | `change_new_refuses_a_broken_policy_when_the_v2_baseline_exists` writes a workflow-v2 baseline and `{` as the policy, and requires `create_change` to fail with `invalid SDD policy` without creating a change directory. |
| REQ-cmd-change-022 | `finalize_commits_an_archive_that_was_left_uncommitted` archives first, then runs finalize, and requires the archive commit. The integration test expects the next action to say to push the archive before merging. |
| REQ-cmd-rules-003 | `an_escaping_principles_path_is_not_a_present_file` requires `/etc/passwd` and `../outside.md` to be rejected as escaping the project. |

- `cargo test --bin specsync -- change_new_refuses_a_broken_policy_when_the_v2_baseline_exists` passed.
- `cargo test --bin specsync -- finalize_commits_an_archive_that_was_left_uncommitted` passed.
- `cargo test --bin specsync -- an_escaping_principles_path_is_not_a_present_file` passed.
