---
change: never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files
artifact: testing
---

# Testing

Fixtures are temporary directories with their own `git init`. Every journal targets files inside the fixture only.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-162 | `src/change_tests/journal_replay_targets.rs`: `a_journal_never_rewrites_or_deletes_git_files`, `a_backslash_entry_cannot_reach_git_files`, `specs_dir_dot_does_not_widen_replay`, `a_journal_git_tracks_is_never_replayed`, `one_entry_outside_the_change_workflow_refuses_the_whole_journal`, `a_bare_array_journal_is_no_longer_replayed`, `a_real_interrupted_write_is_still_recovered`, `a_registry_mapped_spec_is_recovered_and_its_neighbours_are_not`; `tests/integration/project_reads.rs`: `change_new_never_replays_a_committed_journal_into_git` |

Red first on `cf528407`: 5 of the 6 unit tests and the CLI test failed. The replay accepted every planted journal, and `change new` rewrote the fixture's `.git/config`. The control `a_real_interrupted_write_is_still_recovered` passed there, as it should. Three older tests were updated to the new rules: two now write the durable form, and one journals a change record instead of a file at the project root.

Amended before the review: t4c's read of `94e4551d` showed that the rule checked the raw entry while the write turns backslashes into separators, and that `specs_dir = "."` admitted every path. `a_backslash_entry_cannot_reach_git_files` and `specs_dir_dot_does_not_widen_replay` failed on `75dddd76` and pass with the amendment.

Then the full verify lane and the Linux container run.
