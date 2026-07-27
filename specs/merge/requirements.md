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
- Numeric frontmatter `version` fields use `max(ours, theirs)`, accept supported quoted/commented unsigned scalars, and never regress
- Divergent or one-sided non-version frontmatter scalars, nonnumeric versions, and equal numeric versions with different scalar syntax require manual resolution
- Nested mappings, YAML null-versus-list disagreements, and unsupported frontmatter shapes inside a hunk require manual resolution
- Changelog table rows are merged chronologically by date, with deduplication by full row content
- Generic markdown tables union distinct data rows and deduplicate identical rows by first cell; divergent rows with the same key, headers/separators inside the hunk, or a conflicted header whose separator immediately follows the hunk require manual resolution
- Prose section conflicts (like `## Purpose` body text) are never auto-resolved and preserve conflict markers
- `all_files: false` uses `git diff --diff-filter=U` to find only git-conflicted files
- `all_files: true` scans all `.spec.md` files for every conflict-marker family regardless of git state and retains unreadable candidates as explicit findings
- `dry_run: true` returns resolution results without writing any changes to disk
- Unreadable spec files are marked as `Manual` with the read error included in details
- If `git diff` fails in git mode (`all_files: false`), `detect_conflicted_specs` returns no files (the run is a safe no-op); explicit `all_files: true` is the way to scan everything
- Only exact standard opener/base/separator/closer forms are accepted; orphan, nested, duplicate, incomplete, and lookalike markers require manual resolution
- Diff3 base sections are excluded from auto-resolution input and preserved verbatim when a hunk remains manual
- Resolution details name both marker labels and the applied strategy or manual-resolution reason, use `Auto-resolvable` for candidates, and use `Auto-resolved` only after successful persistence
- A file is written only when every hunk resolves safely and the resulting output contains valid frontmatter
- Post-resolution YAML errors, duplicate keys, missing required fields, invalid status, or empty files leave the original file unchanged
- Uniform CRLF/LF style and final-newline presence are preserved
- Results include `spec_path`, `status` (`Resolved` | `Manual` | `Clean`), and `details` for each file
- Human-readable output uses colored formatting to distinguish status types

## Constraints

- Auto-resolution uses a custom parser for its simple top-level key/scalar and known-list subset; candidate output uses the shared checked YAML parser
- Prose sections must never be auto-resolved to prevent loss of important description changes
- Changelog sorting relies on ISO date format (YYYY-MM-DD) for lexicographic ordering
- Resolution strategies are context-aware and cannot be overridden per-file
- Conflict parsing accepts exact standard git markers plus diff3 `||||||| base` sections and fails closed on every marker-like deviation
- Must handle both Windows (`\r\n`) and Unix (`\n`) line endings in conflicted files
- Post-resolution validation must use the same frontmatter parser as the main `parser` module

## Out of Scope

- Interactive merge conflict resolution (TUI or prompts)
- Semantic three-way reconciliation using the base ancestor (diff3 base text is parsed only to keep it out of branch inputs)
- Custom resolution strategies per-project or per-file
- Resolving conflicts in non-spec files (`.rs`, `.md` without spec frontmatter)
- Automatic git add/commit after resolution
- Integration with external merge tools (kdiff3, meld, etc.)
- Visual diff display of changes made during auto-resolution

### REQ-merge-001

The merge engine SHALL resolve only lossless known conflict shapes and SHALL leave ambiguous files untouched with an explicit result.

Acceptance Criteria
- Frontmatter list fields (`files`, `db_tables`, `depends_on`) are unioned and sorted alphabetically when both sides have conflicting values
- Numeric frontmatter `version` fields use `max(ours, theirs)`, accept supported quoted/commented unsigned scalars, and never regress
- Divergent or one-sided non-version frontmatter scalars, nonnumeric versions, and equal numeric versions with different scalar syntax require manual resolution
- Nested mappings, YAML null-versus-list disagreements, and unsupported frontmatter shapes inside a hunk require manual resolution
- Changelog table rows are merged chronologically by date, with deduplication by full row content
- Generic markdown tables union distinct data rows and deduplicate identical rows by first cell; divergent rows with the same key, headers/separators inside the hunk, or a conflicted header whose separator immediately follows the hunk require manual resolution
- Prose section conflicts (like `## Purpose` body text) are never auto-resolved and preserve conflict markers
- `all_files: false` uses `git diff --diff-filter=U` to find only git-conflicted files
- `all_files: true` scans all `.spec.md` files for every conflict-marker family regardless of git state and retains unreadable candidates as explicit findings
- `dry_run: true` returns resolution results without writing any changes to disk
- Unreadable spec files are marked as `Manual` with the read error included in details
- If `git diff` fails in git mode (`all_files: false`), `detect_conflicted_specs` returns no files (the run is a safe no-op); explicit `all_files: true` is the way to scan everything
- Only exact standard opener/base/separator/closer forms are accepted; orphan, nested, duplicate, incomplete, and lookalike markers require manual resolution
- Diff3 base sections are excluded from auto-resolution input and preserved verbatim when a hunk remains manual
- Resolution details name both marker labels and the applied strategy or manual-resolution reason, use `Auto-resolvable` for candidates, and use `Auto-resolved` only after successful persistence
- A file is written only when every hunk resolves safely and the resulting output contains valid frontmatter
- Post-resolution YAML errors, duplicate keys, missing required fields, invalid status, or empty files leave the original file unchanged
- Uniform CRLF/LF style and final-newline presence are preserved
- Results include `spec_path`, `status` (`Resolved` | `Manual` | `Clean`), and `details` for each file
- Human-readable output uses colored formatting to distinguish status types

