---
change: refuse-a-broken-policy-and-commit-the-archive-move
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-108 | `change_new_refuses_a_broken_sdd_policy` writes `{` to `.specsync/sdd.json` and requires `create_change` to fail with `invalid SDD policy`. |
| REQ-cmd-change-021 | `status_reads_the_same_broken_policy_error` requires `configured_principles_file` to return that same error. `change status` returns it before printing a briefing. |
| REQ-cmd-change-022 | `finalize_commits_the_archive_move` requires `git log -1` to be `chore(lifecycle): archive <id>` and porcelain to drop `.specsync/changes/<id>`. |
| REQ-cmd-rules-003 | `rules_says_when_the_named_principles_file_is_missing` requires the summary to name the path, say missing, and say to create the file or clear `principles_file`. |

- `cargo test --bin specsync -- change_new_refuses_a_broken_sdd_policy` passed.
- `cargo test --bin specsync -- status_reads_the_same_broken_policy_error` passed.
- `cargo test --bin specsync -- rules_says_when_the_named_principles_file_is_missing` passed.
- `cargo test --bin specsync -- finalize_commits_the_archive_move` passed.

Opening a change and asking its status fail when `.specsync/sdd.json` cannot be parsed. `specsync rules` warns when the named principles file is not on disk. `change finalize` commits the archive move.
