---
change: a-failed-verification-records-the-findings-that-failed-it
artifact: testing
---

# Testing

Unit tests in `src/change_tests/bounded_failure_output.rs`; integration tests run a real `change check`.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-153 | integration `failed_change_check_output_keeps_the_error_behind_many_warnings`, `failed_strict_change_check_output_records_the_warnings_that_failed_it` (fails without the `--strict` branch), `failed_change_check_records_findings_in_command_evidence`, `change_check_records_the_effective_contract_refusal_on_the_first_run`, `passing_change_check_records_no_output_key`; unit `bounded_failure_output_*`, `command_evidence_written_before_the_output_field_parses_without_output` (a record from an older binary parses with no output) |
