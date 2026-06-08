---
spec: output.spec.md
---

## Key Decisions

- **Pure presentation layer**: `output` only formats and prints data passed to it. It does no config loading, file I/O, or process exit — those belong to the command layer (`src/commands/`). This keeps formatting testable in isolation.
- **Color thresholds**: percentages render green at 100, yellow at ≥80, red below 80 (`print_coverage_line`); summary counts color passed green, warnings yellow, failed red. Coloring uses the `colored` crate.
- **`saturating_sub` for failures**: `print_summary` computes `failed = total.saturating_sub(passed)` after a usize-underflow panic was found when `passed` exceeded `total`. This is regression-guarded by `print_summary_does_not_underflow_when_passed_exceeds_total`.
- **Markdown vs terminal**: `print_check_markdown` / `print_diff_markdown` emit plain Markdown (no ANSI) for PR comments and CI summaries; the line/report printers use color for interactive terminals.

## Files to Read First

- `src/output.rs` — all formatting functions plus inline unit tests
- `src/commands/check.rs` / `src/commands/diff.rs` — callers that compute the data and own exit codes

## Current Status

Fully implemented and stable, with inline unit tests covering `print_summary` (underflow + zero/all-passed) and `print_coverage_line` color boundaries. The `saturating_sub` underflow fix is in place.

## Notes

- This module contains no domain logic — it is the rendering layer for `check`, `coverage`, and `diff`.
