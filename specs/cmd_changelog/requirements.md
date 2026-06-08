---
spec: cmd_changelog.spec.md
---

## User Stories

- As a maintainer, I want a changelog of spec changes between two git refs so I can summarize what specs were added, modified, or removed in a release
- As a release automation author, I want the changelog in text, JSON, or markdown so I can embed it in release notes or feed it to other tooling
- As a CLI user, I want a clear error and non-zero exit when I pass a malformed range so mistakes fail fast

## Acceptance Criteria

- `cmd_changelog(root, range, format)` parses `range` with `changelog::parse_range`; a valid range is `FROM..TO` (e.g. `v0.1..v0.2`, `HEAD~5..HEAD`)
- On an invalid range (missing `..`, empty FROM, or empty TO) it prints an error naming the expected format and exits 1
- On a valid range it loads config and calls `changelog::generate_changelog(root, &config.specs_dir, &from, &to)`
- `Json` prints `format_json`; `Markdown` prints `format_markdown`; `Text`, `Github`, `Table`, and `Csv` all fall through to `format_text`
- Markdown and text output use `print!` (no extra command-injected trailing newline); JSON uses `println!`

## Constraints

- Pure orchestration wrapper: range parsing, git diffing, and formatting all live in the `changelog` module
- Must not panic on an invalid range — print and `process::exit(1)`
- The underlying git diff requires the refs to exist; invalid refs surface as a git failure from the delegate, not a panic here

## Out of Scope

- Range parsing, git diff collection, and report formatting (owned by the `changelog` module)
- Writing the changelog to a file (output goes to stdout)
- Interactive prompts or GUI
