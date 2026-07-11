---
spec: merge.spec.md
---

## User Stories

- As a developer resolving a git merge conflict, I want spec-sync to auto-resolve conflicts in YAML frontmatter so that I don't need to manually merge list fields like `files` and `depends_on`
- As a team member, I want changelog entries from both branches to be merged chronologically so that the history is preserved without manual copy-pasting
- As a developer, I want to see which files were auto-resolved vs which need manual intervention so that I know where to focus my attention
- As a CI operator, I want to run merge conflict detection in dry-run mode so that I can check for unresolved conflicts without modifying files
- As a developer, I want conflict detection to only check files that git reports as conflicted so that I don't waste time scanning unchanged specs
- As a maintainer, I want the merge tool to validate frontmatter after resolution so that auto-resolved specs don't end up with invalid YAML
- As a developer integrating spec-sync into another tool, I want merge results available as JSON so that I can programmatically process the outcomes
- As a developer, I want to scan all spec files for conflict markers regardless of git state so that I can find conflicts even in non-standard workflows

## Acceptance Criteria

- Frontmatter list fields (`files`, `db_tables`, `depends_on`) are unioned and sorted alphabetically when both sides have conflicting values
- Frontmatter scalar fields (like `version`, `status`) use "theirs wins" strategy (latest change takes precedence)
- Changelog table rows are merged chronologically by date, with deduplication by full row content
- Generic markdown tables are merged by first cell (symbol name), with "theirs wins" on conflicts and deduplication
- Prose section conflicts (like `## Purpose` body text) are never auto-resolved and preserve conflict markers
- `all_files: false` uses `git diff --diff-filter=U` to find only git-conflicted files
- `all_files: true` scans all `.spec.md` files for conflict markers regardless of git state
- `dry_run: true` returns resolution results without writing any changes to disk
- Unreadable spec files are marked as `Manual` with the read error included in details
- If `git diff` fails in git mode (`all_files: false`), `detect_conflicted_specs` returns no files (the run is a safe no-op); explicit `all_files: true` is the way to scan everything
- Post-resolution frontmatter validation warnings are printed but don't prevent file writes
- Results include `spec_path`, `status` (`Resolved` | `Manual` | `Clean`), and `details` for each file
- Human-readable output uses colored formatting to distinguish status types

## Constraints

- Must not depend on external YAML libraries — uses custom parser for simple key-value and list fields
- Prose sections must never be auto-resolved to prevent loss of important description changes
- Changelog sorting relies on ISO date format (YYYY-MM-DD) for lexicographic ordering
- Resolution strategies are context-aware and cannot be overridden per-file
- Conflict marker detection looks for standard git markers: `<<<<<<< `, `=======`, `>>>>>>> `
- Must handle both Windows (`\r\n`) and Unix (`\n`) line endings in conflicted files
- Post-resolution validation must use the same frontmatter parser as the main `parser` module

## Out of Scope

- Interactive merge conflict resolution (TUI or prompts)
- Three-way merge with base ancestor analysis
- Custom resolution strategies per-project or per-file
- Resolving conflicts in non-spec files (`.rs`, `.md` without spec frontmatter)
- Automatic git add/commit after resolution
- Integration with external merge tools (kdiff3, meld, etc.)
- Visual diff display of changes made during auto-resolution