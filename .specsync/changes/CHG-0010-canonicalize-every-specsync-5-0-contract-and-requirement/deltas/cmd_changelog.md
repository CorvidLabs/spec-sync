## ADDED

### REQUIREMENT REQ-cmd-changelog-001

The changelog command SHALL validate ranges, delegate Git comparison, and render the selected output format with predictable failures.

Acceptance Criteria
- `cmd_changelog(root, range, format)` parses `range` with `changelog::parse_range`; a valid range is `FROM..TO` (e.g. `v0.1..v0.2`, `HEAD~5..HEAD`)
- On an invalid range (missing `..`, empty FROM, or empty TO) it prints an error naming the expected format and exits 1
- On a valid range it loads config and calls `changelog::generate_changelog(root, &config.specs_dir, &from, &to)`
- `Json` prints `format_json`; `Markdown` prints `format_markdown`; `Text`, `Github`, `Table`, and `Csv` all fall through to `format_text`
- Markdown and text output use `print!` (no extra command-injected trailing newline); JSON uses `println!`
