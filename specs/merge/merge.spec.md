---
module: merge
version: 6
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
| `conflict_hunks` | `content: &str` | `Vec<ConflictHunk>` | Structurally complete opener/separator/closer triples only — stricter than `has_conflict_markers`, which also reports marker-shaped lines such as a Markdown setext underline |
| `document_conflict_hunks` | `body: &str` | `Vec<ConflictHunk>` | `conflict_hunks` over a Markdown document with fenced code blocks blanked, so quoted example markers are not read as corruption |
| `conflict_free_side` | `content: &str, side: ConflictSide` | `String` | Rebuild content keeping only one side of every complete hunk; malformed hunks are re-emitted verbatim |
| `unmerged_paths` | `root: &Path` | `Option<HashSet<String>>` | Paths git reports as unmerged; `None` when git could not answer, never an empty set standing in for "clean" |
| `cached_unmerged_paths` | `root: &Path` | `Option<HashSet<String>>` | Process-lifetime memoized `unmerged_paths`, keyed by repository root |

### Exported Structs/Enums

| Type | Description |
|------|-------------|
| `MergeResult` | Outcome for one spec — `spec_path: String`, `status: MergeStatus`, `details: Vec<String>` |
| `MergeStatus` | Enum: `Resolved` (auto-resolved), `Manual` (needs human), `Clean` (no conflicts), `Unknown` (scan never ran) |
| `ConflictHunk` | One complete hunk — `ours: String`, `theirs: String`, `ours_label: String`, `theirs_label: String` |
| `ConflictSide` | Enum: `Ours` (opener-to-separator text), `Theirs` (separator-to-closer text) |

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
16. `all_files: false` with no answer from git yields a single `Unknown` result naming the missing precondition and exits non-zero; an unperformed scan is never reported as "no conflicts found"
17. `unmerged_paths` returns `None` — not an empty set — whenever git could not answer, so no caller can read an unasked question as an all-clear
18. `conflict_hunks` and `document_conflict_hunks` return only complete opener/separator/closer triples, and `document_conflict_hunks` ignores fenced code blocks, so marker-shaped prose (a setext `<h1>` underline, a documented example) is never reported as a conflict

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

### Scenario: Default merge outside a git repository

- **Given** the project root is not a git repository (or git is unavailable)
- **When** `merge_specs(root, specs_dir, true, false)` is called
- **Then** a single `Unknown` result reports the missing precondition and points at `--all`, and `cmd_merge` exits 1

### Scenario: Documented conflict markers inside a fenced code block

- **Given** a spec body contains a complete conflict triple inside a ``` fence
- **When** `document_conflict_hunks(body)` is called
- **Then** no hunks are returned — quoted example text is not a corrupted document

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Spec file unreadable | Marked as `Manual` with read error in details |
| `git diff` command fails | `detect_conflicted_specs` yields `None`; `merge_specs` returns a single `Unknown` result naming the precondition, and the command exits 1 rather than printing "no conflicts found" |
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
| exports | `conflict_hunks`, `conflict_free_side`, `ConflictSide` to decide whether extraction unioned both sides of a conflict |
| validator | `document_conflict_hunks` for spec bodies and `cached_unmerged_paths` for git's own unmerged list |

## Change Log

| Date | Change |
|------|--------|
| 2026-08-14 | Issue #578: expose `conflict_hunks`, `document_conflict_hunks`, `conflict_free_side`, `unmerged_paths`, and `cached_unmerged_paths` as the shared conflict detector; add `MergeStatus::Unknown` so a scan git never performed stops reporting as "no conflicts found" |
| 2026-07-27 | Issue #427 / CHG-0066: parse exact diff3 markers, use numeric max for versions, reject ambiguous rows/scalars and lossy table/YAML shapes, validate reconstructed frontmatter, report both sides accurately, and preserve all-or-nothing writes |
| 2026-04-10 | Populated requirements.md with user stories, acceptance criteria, constraints, and out-of-scope items |
| 2026-04-06 | Initial spec for v3.3.0 |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-27 | CHG-0066-make-issue-427-spec-merge-resolution-lossless-and-truthful-by-parsing-diff3-base: Make issue 427 spec merge resolution lossless and truthful by parsing diff3 bases, preserving both side labels, selecting the maximum numeric version, unioning list fields, leaving conflicting table rows and scalar fields unresolved, and preserving all-or-nothing writes |
| 2026-08-14 | CHG-0124-a-source-file-or-spec-body-carrying-an-unresolved-merge-conflict-must-be-refused: A source file or spec body carrying an unresolved merge conflict must be refused, because extracting declarations from both sides of a hunk describes source that does not exist |
