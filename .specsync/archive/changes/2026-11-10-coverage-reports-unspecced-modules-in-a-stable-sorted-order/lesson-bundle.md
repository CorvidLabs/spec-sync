# Lesson bundle: coverage-reports-unspecced-modules-in-a-stable-sorted-order

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Coverage reports unspecced modules in a stable sorted order
- **Kind**: BugFix
- **Specs**: validator
- **Paths**: src/validator/tests.rs
- **Acceptance**: compute_coverage_checked returns unspecced_modules sorted, so coverage text and JSON, PR comments and generate's spec order are identical across runs of the same command on the same tree, whatever HashMap order config.modules and manifest.modules iterate in

## Evidence

- Verification commit: `84aabe229abb2f5771ca97b1fef73b447ca66132`
- Base commit: `4fe1e1ed93e00102d1eda73d3cf3e19d5cd53c7f`
- Verified by: `specsync check --spec validator`

## From the change's context.md

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

## From the change's testing.md

# Testing

The unit test `coverage_reports_unspecced_modules_in_sorted_order` builds coverage over eight
configured modules (a `HashMap`). It fails when the sort is removed.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| validator coverage reporting (no requirement change) | unit `coverage_reports_unspecced_modules_in_sorted_order`; coverage/validator/generator bin tests 172/172 and coverage/generate/scaffold integration 110/110 unchanged |

## Where these lessons go

- `specs/validator/validator.3md`
