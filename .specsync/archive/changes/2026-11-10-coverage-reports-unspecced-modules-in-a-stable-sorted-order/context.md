---
change: coverage-reports-unspecced-modules-in-a-stable-sorted-order
artifact: context
---

# Context

Issue #612. `compute_coverage_checked` builds `unspecced_modules` from four blocks. Two of
them iterate `HashMap`s (`config.modules`, `manifest.modules`), and the vector was never
sorted, so any project with two or more configured or manifest modules could get a
different module order from the same command on the same tree. The order reaches text and
JSON coverage, PR comment bodies, and the order `generate` writes specs in.

The vector is now sorted once before the report is built. The set of modules is unchanged.
`find_spec_files` (sorted by path) and the uncovered-file list (biggest first) already have
a defined order.

Authorization: Leif's 2026-11-10 merge rule covers bug fixes that change no requirement.
