---
change: print-archive-restore-steps-that-git-and-specsync-read-literally-with-control-bytes-shown-as-text
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-093 (literal pathspecs, #929) | `the_printed_restore_puts_back_only_the_named_package_even_with_glob_characters`, `the_printed_archiving_commit_lookup_matches_only_the_named_package`, `the_package_probe_reads_the_package_as_one_literal_path` |
| REQ-change-093 (`--` before the id, #930) | `the_printed_status_command_reads_a_leading_hyphen_id_as_the_change_id` |
| REQ-change-093 (control bytes, #931; CHANGE-7.b) | `a_control_byte_in_an_archive_directory_name_is_printed_as_text_that_still_restores_it` |
| REQ-change-093 (unchanged verdicts) | the existing corrupt-archive handoff and recovery tests |

## How it runs

The tests in `src/change_tests/literal_guidance.rs` run what the step prints, through bash and
Git, or parse it with Clap. Locally: `cargo test --bin specsync literal_guidance` style filters
on each name, red before the fix and green after. Hosted CI runs the full suite at the archived
head.
