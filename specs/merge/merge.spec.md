---
module: merge
version: 5
status: stable
files:
  - src/merge.rs
db_tables: []
tracks: [98]
depends_on:
  - specs/parser/parser.spec.md
  - specs/validator/validator.spec.md
---

# Merge

## Purpose

Detects and conservatively auto-resolves git merge conflicts in spec files using context-aware strategies. Only lossless conflict shapes are resolved: known YAML lists are unioned, numeric versions take the maximum value, changelog tables are merged chronologically, and non-conflicting generic table rows are unioned. Ambiguous content leaves the entire file untouched.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `merge_specs` | `root: &Path, specs_dir: &Path, dry_run: bool, all_files: bool` | `Vec<MergeResult>` | Scan for conflicted specs and attempt auto-resolution |
| `has_conflict_markers` | `content: &str` | `bool` | Check for opener, base, separator, or closer marker families, including malformed/orphan forms |
| `print_results` | `results: &[MergeResult], dry_run: bool` | `()` | Print human-readable resolution summary with colored output |
| `results_to_json` | `results: &[MergeResult]` | `String` | Format results as JSON with path, status, and details |

### Exported Structs/Enums

| Type | Description |
|------|-------------|
| `MergeResult` | Outcome for one spec — `spec_path: String`, `status: MergeStatus`, `details: Vec<String>` |
| `MergeStatus` | Enum: `Resolved` (auto-resolved), `Manual` (needs human), `Clean` (no conflicts) |

### Resolution Strategies

| Context | Strategy |
|---------|----------|
| Frontmatter (YAML) | Lists (`files`, `db_tables`, `depends_on`) are unioned and sorted; numeric `version` uses max; divergent or one-sided scalars, null-versus-list disagreements, nested mappings, and unsupported shapes require manual resolution |
| `## Change Log` table | Data rows merge chronologically by date and deduplicate by full row; a header/separator inside the hunk or a header whose separator immediately follows the hunk requires manual resolution |
| Generic markdown table | Distinct data rows are unioned and identical duplicate keys are deduplicated; divergent duplicate-key rows or a header/separator inside or immediately after the hunk require manual resolution |
| Prose / section body | No auto-resolution — conflict markers preserved for manual intervention |

## Invariants

1. `all_files: false` uses `git diff --diff-filter=U` to find only git-conflicted files
2. `all_files: true` scans all spec files for conflict markers regardless of git state
3. Frontmatter list fields are unioned (not replaced) and sorted alphabetically
4. Numeric frontmatter `version` values use `max(ours, theirs)`; nonnumeric versions and divergent or one-sided other scalars require manual resolution
5. Changelog rows are sorted by first cell (ISO date) — lexicographic sorting works correctly
6. Prose conflicts are never auto-resolved — always marked as `Manual`
7. Every post-resolution candidate must contain frontmatter with valid YAML, no duplicate keys, all required fields, a valid status, and non-empty files; failures are `Manual` and are not written
8. `dry_run: true` returns results without writing resolved content to disk
9. Custom YAML parser handles simple key-value and list fields without external YAML library
10. Only exact standard marker forms are accepted; diff3 base sections are parsed but never treated as either branch's content
11. A file is written only when every conflict hunk is safely resolved and the resulting frontmatter is valid
12. Resolution details name both conflict-marker labels and the applied strategy or manual-resolution reason; candidate hunks say `Auto-resolvable` until a file is actually persisted
13. Orphan, duplicate, nested, incomplete, and malformed marker families make the complete file manual
14. Table headers/separators, including a conflicted header immediately followed by a clean separator, and nested frontmatter mappings are never reconstructed by the lossy subset parsers
15. Uniform CRLF/LF form and final-newline presence are preserved

## Behavioral Examples

### Scenario: Auto-resolve frontmatter list conflict

- **Given** ours has `files: [a.rs, b.rs]` and theirs has `files: [b.rs, c.rs]`
- **When** `merge_specs` resolves the conflict
- **Then** result is `files: [a.rs, b.rs, c.rs]` (union, sorted)

### Scenario: Auto-resolve changelog conflict

- **Given** ours added a `2026-03-20` entry, theirs added a `2026-03-25` entry
- **When** `merge_specs` resolves the conflict
- **Then** both entries appear in chronological order, status is `Resolved`

### Scenario: Prose conflict requires manual resolution

- **Given** both sides modified the `## Purpose` section text
- **When** `merge_specs` encounters the conflict
- **Then** conflict markers are preserved, status is `Manual`

### Scenario: Divergent Public API row requires manual resolution

- **Given** both sides contain the same Public API symbol with different row content
- **When** `merge_specs` encounters the conflict
- **Then** neither row is selected, the original file remains unchanged, and the result names both sides and the ambiguity

### Scenario: Diff3 conflict

- **Given** a safely mergeable table conflict includes a `||||||| base` section
- **When** `merge_specs` resolves the conflict
- **Then** only the HEAD and incoming rows are unioned and no base marker or base row appears in the output

### Scenario: Dry run

- **Given** conflicted spec files exist
- **When** `merge_specs(root, specs_dir, true, false)` is called
- **Then** returns `MergeResult` entries with resolution details but does not modify files

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Spec file unreadable | Marked as `Manual` with read error in details |
| `git diff` command fails | Returns no files in git mode; explicit `all_files: true` is required for a scan |
| Post-resolution frontmatter invalid | Marked `Manual`; original file remains unchanged |
| Malformed or incomplete conflict block | Marked `Manual`; original block and file remain unchanged |
| Orphan, nested, duplicate, or non-standard marker | Marked `Manual`; no partial write occurs |
| Table header/separator or nested YAML mapping inside a hunk | Marked `Manual`; original bytes remain authoritative |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| parser | `parse_frontmatter` plus checked YAML/duplicate-key validation for post-resolution safety |
| validator | `find_spec_files` to locate all spec files when `all_files: true` |

### Consumed By

| Module | What is used |
|--------|-------------|
| main | `merge_specs`, `print_results`, `results_to_json` via `cmd_merge` subcommand |

## Change Log

| Date | Change |
|------|--------|
| 2026-07-27 | Issue #427 / CHG-0066: parse exact diff3 markers, use numeric max for versions, reject ambiguous rows/scalars and lossy table/YAML shapes, validate reconstructed frontmatter, report both sides accurately, and preserve all-or-nothing writes |
| 2026-04-10 | Populated requirements.md with user stories, acceptance criteria, constraints, and out-of-scope items |
| 2026-04-06 | Initial spec for v3.3.0 |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-27 | CHG-0066-make-issue-427-spec-merge-resolution-lossless-and-truthful-by-parsing-diff3-base: Make issue 427 spec merge resolution lossless and truthful by parsing diff3 bases, preserving both side labels, selecting the maximum numeric version, unioning list fields, leaving conflicting table rows and scalar fields unresolved, and preserving all-or-nothing writes |
