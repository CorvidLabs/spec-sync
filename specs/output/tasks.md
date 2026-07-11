---
spec: output.spec.md
---

## Post-5.0 Test Debt

- [ ] No direct unit tests for `print_coverage_report` / `print_check_markdown` / `print_diff_markdown` — covered only indirectly via integration fixtures (`diff_human_readable_output`, `score_*`)

## Done

- [x] `print_summary` / `print_coverage_line` / `print_coverage_report` colored terminal output
- [x] `print_check_markdown` and `print_diff_markdown` for PR comments and CI summaries
- [x] `saturating_sub` underflow fix in `print_summary` (`passed > total` no longer panics)
- [x] Inline unit tests for `print_summary` (underflow, zero/all-passed) and `print_coverage_line` color boundaries
- [x] Initial spec creation with all required sections
- [x] Requirements and acceptance criteria documented

## Open


## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
