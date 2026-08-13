---
change: CHG-0109-a-symlink-under-a-source-directory-must-be-skipped-and-disclosed-never-abort-di
artifact: docs
---

# Docs

## CHANGELOG

One `Fixed` entry: a symlink under a source directory no longer aborts `check`, `coverage`,
`score`, and `generate`. It is skipped, disclosed in text, markdown, and JSON, and gates
under `--strict`.

## Behaviour change

| tree | before | after |
|---|---|---|
| no symlinks under source dirs | — | unchanged in every respect |
| symlink under a source dir | single-line abort, exit 1 | run completes; exclusion disclosed; bare `check` exit 0, `--strict` exit 1 |
| symlink escaping the project root | abort | never traversed, content never read, disclosed as a skip |
| configured `source_dirs` entry that is a symlink | abort | still aborts, deliberately |
| symlink in the spec tree | abort | still aborts, deliberately |

## New public API

| Symbol | Spec |
|---|---|
| `CoverageReport::skipped_links` | `specs/types` |
| `output::print_skipped_links` | `specs/output` |

## Note for consumers of coverage numbers

Coverage percentages are computed over what was measured. When entries are skipped the
denominator shrinks with them, which is why the exclusion is printed next to the figures
rather than in a separate section, and why `--strict` refuses to call such a tree clean.
A dashboard reading the JSON should treat a non-empty `skipped_links` as qualifying the
percentage.
