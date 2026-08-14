---
change: CHG-0125-every-output-format-must-report-the-same-set-of-findings-so-a-machine-readable
artifact: tasks
---

# Tasks

1. Finding list and renderers in `output.rs`; every format draws from it.
2. One coverage payload constructor; route CLI and both MCP surfaces through it.
3. De-duplicate `csv_field`.
4. Merge staleness findings at EVERY non-text arm, not the tabular pair.
5. Parity test across all formats; widen the staleness loop to all four.
