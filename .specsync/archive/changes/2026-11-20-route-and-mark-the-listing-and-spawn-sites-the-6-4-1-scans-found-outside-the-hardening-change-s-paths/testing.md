---
change: route-and-mark-the-listing-and-spawn-sites-the-6-4-1-scans-found-outside-the-hardening-change-s-paths
artifact: testing
---

# Testing

| Case | Test |
|------|------|
| No unmarked listing | `tests/integration/project_reads.rs`: `no_project_file_is_read_outside_the_bounded_read` |
| No unmarked Git `config` call | `tests/integration/project_writes.rs`: `no_spawned_git_command_writes_files_unmarked` |
| No unmarked spawn | `tests/integration/project_writes.rs`: `no_spawned_command_other_than_git_is_unmarked` |
| `--template` FIFO | `tests/integration/hardening_641.rs`: `scaffold_does_not_wait_on_a_fifo_template` |
