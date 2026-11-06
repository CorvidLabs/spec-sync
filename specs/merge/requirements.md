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
- If `git diff` fails in git mode (`all_files: false`), `detect_conflicted_specs` returns `None`, meaning unknown and never an empty set, and `merge_specs` reports one `Unknown` result that names the missing precondition and points at `--all`; nothing is written and `merge` exits non-zero (REQ-cmd-merge-002). Explicit `all_files: true` is the way to scan everything
- Only exact standard opener/base/separator/closer forms are accepted; orphan, nested, duplicate, incomplete, and lookalike markers require manual resolution
- Diff3 base sections are excluded from auto-resolution input and preserved verbatim when a hunk remains manual
- Resolution details name both marker labels and the applied strategy or manual-resolution reason, use `Auto-resolvable` for candidates, and use `Auto-resolved` only after successful persistence
- A file is written only when every hunk resolves safely and the resulting output contains valid frontmatter
- Post-resolution YAML errors, duplicate keys, missing required fields, invalid status, or empty files leave the original file unchanged
- Uniform CRLF/LF style and final-newline presence are preserved
- Each result includes `spec_path` (JSON `path`), `status` (`Resolved` | `Manual` | `Clean` | `Unknown`), and `details`. `Resolved`, `Manual` and `Clean` each describe one examined spec file; `Unknown` is a single scan-level result whose `spec_path` is the specs directory, returned when the git-mode scan could not run
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
- If `git diff` fails in git mode (`all_files: false`), `detect_conflicted_specs` returns `None`, meaning unknown and never an empty set, and `merge_specs` reports one `Unknown` result that names the missing precondition and points at `--all`; nothing is written and `merge` exits non-zero (REQ-cmd-merge-002). Explicit `all_files: true` is the way to scan everything
- Only exact standard opener/base/separator/closer forms are accepted; orphan, nested, duplicate, incomplete, and lookalike markers require manual resolution
- Diff3 base sections are excluded from auto-resolution input and preserved verbatim when a hunk remains manual
- Resolution details name both marker labels and the applied strategy or manual-resolution reason, use `Auto-resolvable` for candidates, and use `Auto-resolved` only after successful persistence
- A file is written only when every hunk resolves safely and the resulting output contains valid frontmatter
- Post-resolution YAML errors, duplicate keys, missing required fields, invalid status, or empty files leave the original file unchanged
- Uniform CRLF/LF style and final-newline presence are preserved
- Each result includes `spec_path` (JSON `path`), `status` (`Resolved` | `Manual` | `Clean` | `Unknown`), and `details`. `Resolved`, `Manual` and `Clean` each describe one examined spec file; `Unknown` is a single scan-level result whose `spec_path` is the specs directory, returned when the git-mode scan could not run
- Human-readable output uses colored formatting to distinguish status types

### REQ-merge-002

Conflict detection SHALL be available as a pure content predicate and as a git-status query.

Acceptance Criteria
- The content predicate takes text and needs no repository.
- The git query distinguishes "unknown" from "no unmerged paths": a failed or absent git reports unknown, never clean.
- Conflict-marker syntax inside a fenced code block is not a conflict; blanking preserves line numbers so diagnostics still point at the real body.

### REQ-merge-003

`unmerged_paths` SHALL spawn git through `crate::git_utils::git_cmd`, so a `check` or MCP caller that only asks for unmerged paths does not forward host secrets or git-override variables.

Acceptance Criteria
- The git child starts with a cleared environment and the `git_utils` inherit allowlist; `GITHUB_TOKEN`, `AWS_SECRET_ACCESS_KEY`, and `SPECSYNC_TEST_SECRET` are absent from that child.
- `None` vs empty-set unknown-vs-clean contract is unchanged.
- `cached_unmerged_paths` inherits the sanitization because it calls `unmerged_paths`.

### REQ-merge-004

The git merge driver SHALL merge two independent edits of one module spec so `version:`, the change-log rows, and `files:` do not conflict, and the version SHALL count every numeric bump from the common ancestor.

Acceptance Criteria
- Two branches that each bump the same integer version and append a different change-log row and `files:` entry merge in either order with one `version:` line, both rows, and both paths.
- The merged version is the ancestor plus both deltas, and it is never lower than either side.
- A one-sided edit is returned byte-for-byte.
- The marker resolver still uses `max(version)` and still leaves prose hunks manual.

### REQ-merge-005

Divergent prose in a spec section SHALL remain a conflict, and a requirements file SHALL keep both bodies when two branches add the same new requirement ID.

Acceptance Criteria
- A `## Purpose` paragraph edited differently on both sides stays a conflict and keeps both texts.
- Requirement blocks added under different IDs are both present after either merge order.
- The same new requirement ID with two different bodies is kept twice.
- An existing requirement edited differently on both sides stays a conflict.

