## MODIFIED

### REQUIREMENT REQ-merge-001

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
