---
change: report-states-the-require-coverage-verdict-when-another-finding-decides-the-exit
artifact: testing
---

# Testing

Integration tests in `tests/integration/regression_w1.rs` run the built binary on the
minimal project with and without git history. The verdict test fails on the previous
`report.rs`.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-cmd-report-003 | integration `report_states_the_coverage_verdict_when_staleness_decides_the_exit`, `report_prints_no_extra_coverage_verdict_when_the_gate_decides_the_exit`, `report_require_coverage_above_actual_exits_1`, `report_require_coverage_at_actual_exits_0`; the staleness_unmeasurable report tests are unchanged and pass |
