---
spec: util.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/util.rs` | cargo test util:: | `test_levenshtein`, `test_safe_regex_valid`, `test_safe_regex_invalid` |

## Coverage Gaps

- Integration gap: add a fixture for "Suggest nearby filenames" before changing user-visible CLI output, generated files, or error handling in util.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Suggest nearby filenames | the strings `config.ts` and `confg.ts` | `levenshtein` compares them | it returns `1`, allowing validation to suggest the near miss |
| Reject invalid regex | an invalid pattern such as `[invalid` | `safe_regex` tries to compile it | it returns `None` instead of propagating a regex parser error |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Empty string passed to `levenshtein` | Returns the character length of the other string | Keep or add a focused assertion before changing this behavior |
| Invalid regex syntax | `safe_regex` returns `None` | Keep or add a focused assertion before changing this behavior |
| Pattern exceeds configured regex size limits | `safe_regex` returns `None` | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/util.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.

## Symbolic links (REQ-util-002)

Every fixture is one temporary directory with the project and an `outside` directory in it, so no link in a test reaches a real file. The link tests are Unix-only.

| Case | Required behavior | Test |
|------|-------------------|------|
| A path this module writes leaves the project through a link, dangles, or is not a regular file | Refused with an error naming the path; nothing outside the project changes | `src/util.rs`: `project_file_functions_refuse_a_link_that_leaves_the_project`, `project_file_functions_refuse_a_dangling_link_and_a_chain_that_leaves`, `project_file_functions_follow_a_link_that_stays_inside_the_project`, `project_file_functions_refuse_a_directory_that_leaves_and_a_path_outside`, `project_file_functions_create_replace_read_and_remove_plain_files` |

## Bounded reads (REQ-util-003)

FIFOs run under a timeout and over-bound files are sparse; nothing reads `/dev/zero`. Unix-only.

| Case | Required behavior | Test |
|------|-------------------|------|
| A project file this module reads is a FIFO, a link out of the project, or over its bound | Refused without waiting or reading past the bound | `src/util.rs`: `project_file_reads_refuse_what_they_must_not_read`, `project_file_reads_hold_the_bound`, `project_file_reads_follow_inside_links_and_keep_the_std_contracts`; `tests/integration/project_reads.rs`: `no_project_file_is_read_outside_the_bounded_read`, the source scan that fails on any direct file read outside the bounded project read |

## Work-tree confinement and the Git directory (REQ-util-004, REQ-util-005)

Every link points inside the test's own temporary directory.

| Case | Required behavior | Test |
|------|-------------------|------|
| A monorepo subproject's link stays inside its repository; one leaves it | Followed; refused | `tests/integration/project_writes.rs`: `a_subproject_writes_through_a_link_that_stays_inside_its_repository`, `a_subproject_reads_specs_through_a_link_that_stays_inside_its_repository`, `a_subproject_link_that_leaves_its_repository_is_still_refused` |
| An absolute link, even to a file inside | Refused as an absolute link | `tests/integration/project_writes.rs`: `an_absolute_link_is_refused_as_an_absolute_link` |
| A path or link into `.git`, in any case, or into a Git directory with another name | Refused as leading into the Git directory; the hook or config keeps its bytes | `src/util.rs`: `project_file_functions_refuse_the_git_directory`, `project_file_functions_refuse_a_git_directory_under_any_name`; `tests/integration/project_writes.rs`: `hooks_install_never_writes_through_a_link_into_the_git_directory`, `change_answer_never_writes_change_md_into_a_hook` |
| A FIFO at a path being written | Refused without blocking | `src/util.rs`: `writing_a_fifo_is_refused_without_blocking` |
| The new helpers: reading within a named directory, removing an empty directory, walking a project directory or the root itself | Stay inside; never follow a link out; the walk lists file links for the reader and stops at its budget | `src/util.rs`: `reading_within_a_named_directory_stays_inside_it`, `removing_an_empty_directory_never_follows_a_link_out`, `finding_project_files_never_follows_a_link`, `finding_project_files_from_the_root_itself_lists_its_files` |
| Any new direct file write outside the helpers | The source scan fails | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |

## Local state, the enclosing work tree and the scans (REQ-util-006, REQ-util-007, REQ-util-008)

Every link points inside the test's own temporary directory.

| Case | Required behavior | Test |
|------|-------------------|------|
| A project in a Git work tree, a subproject beside it, and a project with no Git | State under the Git directory, one folder per project; with no Git, under the user's cache folder; nothing in the work tree | `src/util/local_state.rs`: `local_state_lives_outside_the_work_tree` |
| A link where SpecSync makes its folder, a FIFO, an oversized file and a link in the folder | Refused without blocking; a link is unlinked, never followed | `src/util/local_state.rs`: `local_state_refuses_links_fifos_and_oversized_files` |
| An untracked project unpacked inside another work tree | Confined to itself: its link to the outer work tree's file is refused and the file keeps its bytes | `tests/integration/project_writes.rs`: `an_untracked_project_inside_another_work_tree_is_confined_to_itself` |
| A tracked monorepo subproject | Still follows a link inside its repository | `tests/integration/project_writes.rs`: the three `a_subproject_*` tests |
| A production read in an `include!`d `tests.rs`, braces in literals, cap-std calls, Git arguments that write | Reported until bounded or marked | `tests/integration/project_reads.rs`: `the_scan_lexer_blanks_only_comments_and_literals`, `no_project_file_is_read_outside_the_bounded_read`; `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers`, `no_spawned_git_command_writes_files_unmarked` |

## The user cache folder only (REQ-util-006, change 7)

| Case | Required behavior | Test |
|------|-------------------|------|
| A Git work tree | The state folder is in the user's cache folder, not under `.git` | `src/util/local_state.rs`: `local_state_never_lives_in_a_git_directory`, `local_state_lives_outside_the_work_tree` |
| A copy of a project with its `.git`, cache included | Every spec is validated | `tests/integration/repository_inputs.rs`: `check_never_uses_a_hash_cache_an_archive_ships_in_its_git_directory` |
| `XDG_CACHE_HOME` inside the project | Refused: one warning, every spec validated | `tests/integration/repository_inputs.rs`: `check_refuses_a_user_cache_folder_inside_the_project` |
| A project folder named `*` inside another work tree | Confined to itself | `tests/integration/project_writes.rs`: `a_project_folder_named_like_a_glob_is_still_confined_to_itself` |
