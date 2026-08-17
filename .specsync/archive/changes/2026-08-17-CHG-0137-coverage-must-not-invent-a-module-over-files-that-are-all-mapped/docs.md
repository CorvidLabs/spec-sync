---
change: CHG-0137-coverage-must-not-invent-a-module-over-files-that-are-all-mapped
artifact: docs
---

# Docs

A CHANGELOG entry lands under `## [Unreleased]` → `### Fixed`.

## What it must state, and does

- **What a user saw:** `Modules without specs (1): strutil/` — a trailing-slash directory that
  does not exist — printed beside `5/5 (100%)` file coverage at exit 0.
- **That it reproduces on this repository**, naming `specsync` and `change_tests` and where each
  came from. A reader who runs `coverage` here will see it, and a report that did not say so
  would leave them wondering whether their own project was misconfigured.
- **The mechanism:** four derivations asked whether a spec *directory* of that name existed, and
  none consulted the file mapping computed thirty lines above.
- **That all four were fixed**, and that the one producing the phantom here was not the one the
  bug report named.
- **What did not change:** the coverage percentages. This is the detail that tells a reader the
  fix removed a false claim rather than widening what counts as covered.

## Not documented here, deliberately

Four adjacent defects were found while fixing this and filed rather than folded in: #611
(`generate` can double-map files), #612 (nondeterministic module order), #613 (excluded files
still name modules), and #610 (`has_spec` is a hardcoded constant in the JSON payload). The
CHANGELOG entry does not describe them, because they are not fixed; they are tracked.

No user-facing guide describes the `modules` list, and `--help` text is unchanged, so nothing
outside the CHANGELOG requires updating.
