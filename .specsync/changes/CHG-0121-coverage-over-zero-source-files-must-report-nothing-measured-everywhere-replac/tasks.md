---
change: CHG-0121-coverage-over-zero-source-files-must-report-nothing-measured-everywhere-replac
artifact: tasks
---

# Tasks

1. Remove the two `usize` fields; add `Option`-returning accessors with a single
   shared helper for the zero-denominator question.
2. Fix every resulting compile error by deciding, per site, what that surface
   renders when nothing was measured.
3. Route `src/output.rs` through the accessors so the one previously-correct
   renderer stops being a tenth implementation.
4. `--require-coverage` fails closed on `None`.
5. Regression covering every command x every format x both MCP surfaces.
6. CHANGELOG entry.
