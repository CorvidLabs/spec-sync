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
