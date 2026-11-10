---
change: bounded-failure-output-never-splits-a-multi-byte-character-and-gate-refusal-evidence-is-tested
artifact: testing
---

# Testing

Unit tests in `src/change_tests/bounded_failure_output.rs` call `bounded_failure_output`
directly. The multi-byte test fails on the previous code with "byte index 3 is not a char
boundary". The integration test runs `change check` against a fixture whose declared
module drifts under strict enforcement.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-151 | unit `bounded_failure_output_respects_the_cap_at_a_char_boundary`, `bounded_failure_output_starts_on_a_line_boundary`, `bounded_failure_output_under_the_cap_is_verbatim`; integration `change_check_records_the_effective_contract_refusal_on_the_first_run` (gate-refusal `output`), `failed_change_check_records_findings_in_command_evidence` and `passing_change_check_records_no_output_key` from round 1 |
