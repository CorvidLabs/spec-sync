---
change: never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound
artifact: testing
---

# Testing

Every fixture is a temporary directory holding the project and an `outside` directory, so no link reaches a real file. FIFO tests run the command under a timeout, so a run on the unfixed code fails instead of hanging. Over-bound files are sparse, so they cost no disk. Nothing reads `/dev/zero`. The link and FIFO tests are Unix-only.

| Test | Shows | On `3a02a5d5` |
|---|---|---|
| `src/util.rs` `project_file_reads_*` | the bounded read: link out, dangling, FIFO, directory, over-bound, at-bound, inside link, missing | does not compile (the bound and the two functions are new) |
| `tests/integration/project_reads.rs` FIFO cases | each command finishes without waiting | times out |
| the same file, outside-link cases | the outside bytes are not used, and the message names the link | the outside file is read |
| the same file, over-bound cases | the read is refused naming the bound | the file is read whole |
| `no_project_file_is_read_outside_the_bounded_read` | no direct project file read remains | about 140 sites found |

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-util-003 | `src/util.rs`: `project_file_reads_refuse_what_they_must_not_read`, `project_file_reads_hold_the_bound`, `project_file_reads_follow_inside_links_and_keep_the_std_contracts`; the source scan `no_project_file_is_read_outside_the_bounded_read` |
| REQ-change-161 | `tests/integration/project_reads.rs`: `change_status_does_not_wait_on_a_policy_that_is_not_a_regular_file`, `change_status_does_not_read_a_policy_linked_outside_the_project`, `change_status_refuses_a_policy_over_its_bound`, `a_lifecycle_write_does_not_wait_on_or_follow_the_transaction_journal`, `a_policy_linked_inside_the_project_is_still_read`; the source scan `no_project_file_is_read_outside_the_bounded_read` |
| REQ-cmd-change-038 | `change_audit_does_not_wait_on_the_policy`; the source scan `no_project_file_is_read_outside_the_bounded_read` |
| REQ-config-016 | `check_does_not_wait_on_a_config_file_that_is_not_a_regular_file`; the source scan `no_project_file_is_read_outside_the_bounded_read` |
| REQ-ignore-002 | `check_does_not_wait_on_an_ignore_file_that_is_not_a_regular_file`; the source scan `no_project_file_is_read_outside_the_bounded_read` |
| REQ-hash-cache-008 | `check_does_not_wait_on_a_companion_that_is_not_a_regular_file`, `check_refuses_a_hash_cache_over_its_bound`; the source scan `no_project_file_is_read_outside_the_bounded_read` |
| REQ-validator-049 | `check_does_not_read_requirements_linked_outside_the_project`; the source scan `no_project_file_is_read_outside_the_bounded_read` |
| REQ-exports-011 | `check_does_not_read_exports_from_a_source_linked_outside_the_project`; the source scan `no_project_file_is_read_outside_the_bounded_read` |
| REQ-mcp-012 | `mcp_diff_does_not_wait_on_a_lock_that_is_not_a_regular_file`; the source scan `no_project_file_is_read_outside_the_bounded_read` |
| REQ-agents-012 | `agents_install_does_not_wait_on_a_manifest_that_is_not_a_regular_file`; the source scan `no_project_file_is_read_outside_the_bounded_read` |
| REQ-archive-002 | `archive_tasks_does_not_wait_on_a_tasks_file_that_is_not_a_regular_file`; the source scan `no_project_file_is_read_outside_the_bounded_read` |
| REQ-hooks-007 | `hooks_status_does_not_wait_on_a_hook_file_that_is_not_a_regular_file`; the source scan `no_project_file_is_read_outside_the_bounded_read` |
| REQ-cmd-check-019 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-cmd-diff-005 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-cmd-init-009 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-cmd-lifecycle-005 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-cmd-merge-005 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-cmd-migrate-006 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-commands-014 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-cmd-report-006 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-cmd-resolve-003 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-cmd-stale-005 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-compact-002 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-deps-004 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-generator-011 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-merge-007 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-output-007 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-registry-004 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-schema-002 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-scoring-007 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-view-002 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |

Red first on `3a02a5d5`, with only the tests added: 11 of the 17 integration tests failed, among them the source scan with 187 direct reads. The 6 that passed are guards that already held on base: the confined discovery for a FIFO companion and for linked-out source and requirements files, `archive-tasks`' own preflight, change 2's agents manifest read, and the inside-link control. The `util` unit tests use the new bound and functions, so they could not run on base.

Then the full `fledge lanes run verify` lane locally, and `cargo test` in a local Linux container, since the private fork runs no CI.
