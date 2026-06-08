---
spec: cmd_score.spec.md
---

## Key Decisions

- `cmd_score` is a pure renderer over the `scoring` module: it scores each discovered spec with `score_spec`, aggregates with `compute_project_score`, then formats per the requested `OutputFormat`.
- Four output renderers live here — text, table (ASCII), CSV (with a SUMMARY row), and JSON — each driven off the same `ProjectScore`.
- Scoring is informational: unlike `check`, it never sets a non-zero exit code.
- "Batch mode" (no filters or `--all`) prints a progress header in text mode only, so CSV/JSON stay clean for piping.
- `--explain` deepens output: in text it lists every criterion with ✓/✗ and points; in JSON it adds an `explain` array; in table it adds the five sub-score columns.

## Files to Read First

- `src/commands/score.rs` — `cmd_score` plus the `print_text_output`, `print_table_output`, `print_csv_output`, and colorize helpers
- `src/scoring.rs` — `score_spec`, `compute_project_score`, `SpecScore`, `ProjectScore`, and the per-dimension `explain` data
- `src/commands/mod.rs` — `load_and_discover`, `filter_specs`, `filter_by_status`
- `src/types.rs` — `OutputFormat`

## Current Status

Fully implemented and stable. Covered end-to-end by `tests/integration.rs` (text, JSON, table, CSV, MCP, and with/without `--all`). No inline `#[cfg(test)]` module in `score.rs` itself.

## Notes

- The live `cmd_score` signature also takes `all: bool`, `exclude_status`, and `only_status` beyond what the spec's API table lists.
- Orchestrates the scoring module; the grading rubric is defined there, not here.
