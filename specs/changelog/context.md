---
spec: changelog.spec.md
---

## Key Decisions

- Spec state is read from git (`git ls-tree` + `git show`) rather than the working tree, so the changelog never depends on or disturbs uncommitted edits.
- A spec is only "modified" when raw content differs AND a tracked frontmatter field or `##` section actually changed — identical-content or no-op edits are dropped.
- Section diffing is coarse: top-level `##` sections are reported as `(added)`/`(removed)`/`(modified)`, never line-by-line, to keep output reviewable.
- The `ChangelogReport` model is shared by all three formatters (text/JSON/markdown) so analysis and rendering stay decoupled.
- Output is sorted by module name for deterministic, reviewable diffs.

## Files to Read First

- `src/changelog.rs` — entire module (types, git helpers, `compare_frontmatter`, `compare_sections`, `generate_changelog`, formatters)
- `src/parser.rs` — `parse_frontmatter` (the parse used at each ref)
- `src/types.rs` — `Frontmatter` fields that drive field-level diffing
- `src/commands/changelog.rs` — the CLI wiring that calls `generate_changelog` and selects a formatter

## Current Status

Stable. Frontmatter and section diffing, range parsing, and all three formatters are implemented and unit-tested (including git-backed `generate_changelog` tests via a temp repo). No CLI-level integration test yet.
