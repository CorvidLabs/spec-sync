## ADDED

### REQUIREMENT REQ-merge-001

The merge engine SHALL resolve only lossless known conflict shapes and SHALL leave ambiguous files untouched with an explicit result.

Acceptance Criteria
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
