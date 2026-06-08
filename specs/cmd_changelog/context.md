---
spec: cmd_changelog.spec.md
---

## Key Decisions

- Thin command wrapper: parse range, load config, call `changelog::generate_changelog`, then dispatch on `OutputFormat`. No domain logic here.
- Range validation happens up front via `parse_range`; failure prints the expected `FROM..TO` format and exits 1 before any config is loaded.
- Format dispatch collapses `Text`/`Github`/`Table`/`Csv` onto `format_text` — only Json and Markdown get dedicated renderers.

## Files to Read First

- `src/commands/changelog.rs` — the command wrapper (this module)
- `src/changelog.rs` — `parse_range`, `generate_changelog`, `format_text`/`format_json`/`format_markdown`, and `ChangelogReport`
- `src/types.rs` — `OutputFormat`

## Current Status

Implemented and stable. The `changelog` delegate is heavily unit-tested (range parsing, frontmatter/section diffing, all three formatters, end-to-end generation). The wrapper itself has no inline tests.

## Notes

- Output is emitted with `print!` for text/markdown so the formatter controls trailing newlines; JSON uses `println!`.
