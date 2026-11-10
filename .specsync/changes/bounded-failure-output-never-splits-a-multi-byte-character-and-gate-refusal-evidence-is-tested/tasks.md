---
change: bounded-failure-output-never-splits-a-multi-byte-character-and-gate-refusal-evidence-is-tested
artifact: tasks
---

# Tasks

- [x] Walk the 4 KiB cut forward to a character boundary in `bounded_failure_output`
- [x] Unit tests: multi-byte character at the cut, line-boundary tail, verbatim under the cap
- [x] Assert the gate-refusal entry's `output` in `change_check_records_the_effective_contract_refusal_on_the_first_run`
- [x] List `src/change_tests/bounded_failure_output.rs` in the change spec's `files:`
