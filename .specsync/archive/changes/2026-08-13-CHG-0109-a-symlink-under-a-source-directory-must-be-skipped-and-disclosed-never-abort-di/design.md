---
change: CHG-0109-a-symlink-under-a-source-directory-must-be-skipped-and-disclosed-never-abort-di
artifact: design
---

# Design

Two halves, and the second is the one that makes the first safe: discovery stops aborting,
and every exclusion is disclosed on every channel.

## 1. Collect instead of abort

`CoverageTraversalBudget` already threads through every discovery and coverage walk, so it
is the natural carrier and no signature changes were needed. It gains:

```rust
skipped_links: BTreeSet<String>,
```

`BTreeSet` rather than `Vec`: the same link is reachable from both the source-detection
scan and the coverage walk, and the report must be deterministic.

`record_skipped_link` normalizes to forward slashes so the reported path is stable across
platforms.

Three sites change from `return Err(...)` to record-and-continue:

| site | function |
|---|---|
| source-detection scan, root level | `retained_source_dirs_by_scan` |
| source-detection scan, nested | `retained_directory_contains_source` |
| the coverage walk | `snapshot_coverage_directory` |

## 2. Disclosure, on every channel

`CoverageReport` gains `skipped_links: Vec<String>`, populated from the budget.

| channel | how |
|---|---|
| text | `print_skipped_links`, called from `print_coverage_line` |
| markdown | an `**Excluded (symlinks, not traversed):**` bullet in the Coverage section |
| JSON | a `skipped_links` array |

All three deliberately sit **with the coverage figures**, not in a separate findings
section. These percentages can only be read honestly next to what was excluded from them,
and a reader who sees the numbers must see the exclusion in the same glance.

JSON matters most: machine consumers are exactly who cannot see the prose, and they are the
ones acting on `passed`.

Text and markdown name at most `SKIPPED_LINK_DISPLAY_LIMIT` (5) paths and summarize the
remainder; JSON carries the full list.

## 3. `--strict` gates

Both exit paths — `compute_exit_code` (JSON/markdown) and `exit_with_status` (text) —
return 1 under `--strict` when any link was skipped, naming the count.

Bare `check` stays exit 0. The exclusion is a fact to report, not an error; `--strict` is
where a caller says it will not accept a partially-measured tree.

This mirrors the `--require-coverage` vacuous-pass guard immediately below it, which
already refuses to let a gate pass over zero measured files for the same reason.

## What deliberately still fails loudly

| case | why |
|---|---|
| a symlink that **is** a configured `source_dirs` entry | skipping something discovery merely encountered loses nothing; silently skipping a source tree the author asked to be measured is the failure this change prevents |
| a symlink in the **spec** tree | skipping a symlinked spec drops a whole spec from validation, unnoticed — a much larger hole than a skipped source file, whose real path is walked anyway |

## Public API added

| Symbol | Module |
|---|---|
| `CoverageReport::skipped_links` | `types` |
| `output::print_skipped_links` | `output` |
