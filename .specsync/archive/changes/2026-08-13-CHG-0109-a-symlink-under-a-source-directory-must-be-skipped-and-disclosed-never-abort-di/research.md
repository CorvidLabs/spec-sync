---
change: CHG-0109-a-symlink-under-a-source-directory-must-be-skipped-and-disclosed-never-abort-di
artifact: research
---

# Research

## The rejection sites

Seven sites call `coverage_metadata_is_link` and return `Err`. Which one fires depends on
configuration, which is why the issue reproduced with two different messages:

| line | function | message | in scope |
|---|---|---|---|
| 424 | `retained_source_dirs_by_scan` | `Coverage source-detection path` | yes — hit by the no-config repro |
| 490 | `retained_directory_contains_source` | `Coverage source-detection path` | yes |
| 1045 | `snapshot_coverage_directory` | `Coverage source` | yes — hit by the with-config repro |
| 1251 | `open_retained_coverage_directory` | `Coverage source directory` | **no** — configured entry |
| 714, 4935, 4967 | spec-tree walks | `Coverage spec path` / `spec-module path` | **no** |

Site 1251 was initially converted and then reverted. It opens a path the project
*configured* as a `source_dirs` entry, not one discovery happened to meet. It also has no
budget in scope and seven callers, only some of which could supply one — the awkward
threading was the prompt to look harder at whether it belonged in scope at all. It does
not: silently skipping a configured source tree drops everything the author asked to have
measured.

## Carrier

`CoverageTraversalBudget` is already threaded through every in-scope walk, including via
`CoverageSourceAccumulator.budget`. Putting the collector there required **no signature
changes**, which is what made the three-site conversion small enough to reason about.

## Two defects found while implementing, both mine

**The first working version reported `File coverage: 1/1 (100%)` with the skipped link
unmentioned.** The run no longer aborted, so "it works" was tempting. It was the exact
denominator hazard the design had been written to prevent. Caught by re-reading the
acceptance criteria rather than by re-running the repro.

**`--strict` silently kept passing after the gate was added.** Exit logic is duplicated in
`compute_exit_code` (JSON/markdown) and `exit_with_status` (text); only the first had been
patched. It was invisible until the fixture was reduced to one with no other warnings,
because the pre-existing warning made `--strict` exit 1 for an unrelated reason. Worth
filing separately: two functions encoding the same policy will drift.

## Two more, caught by the suite and the drill

**`cargo build --release` went green while eleven `#[cfg(test)]` construction sites of
`CoverageReport` lacked the new field.** A plain build does not compile test code. This is
already a recorded rule; it was ignored in favour of the faster command.

**Two drill assertions passed against the pre-fix binary.** The abort message happens to
contain the skipped link's path, so `grep src/alias.py` matched it; and the abort exits
non-zero, so a bare `exit != 0` satisfied the `--strict` assertion. Both were rewritten to
require evidence the walk completed. A drill that passes on a broken binary is worse than
no drill, because it converts "untested" into "believed tested".

## Verified escape behaviour

A symlinked directory pointing outside the project root: the run completes, the link is
disclosed, and the outside file's content never appears in the output. All 48 symlink tests
in the suite continue to pass, including `safe_project_paths_reject_symlink_escapes`.
