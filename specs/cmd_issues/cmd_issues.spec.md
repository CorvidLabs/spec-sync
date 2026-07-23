---
module: cmd_issues
version: 7
status: stable
files:
  - src/commands/issues.rs
db_tables: []
tracks: []
depends_on:
  - specs/commands/commands.spec.md
  - specs/config/config.spec.md
  - specs/github/github.spec.md
  - specs/parser/parser.spec.md
  - specs/types/types.spec.md
  - specs/validator/validator.spec.md
  - specs/ignore/ignore.spec.md
---

# Cmd Issues

## Purpose

Implements the `specsync issues` command — verifies GitHub issue references in spec frontmatter (`implements:`, `tracks:` fields) against the GitHub API. Reports valid, closed, not-found, and errored references. Optionally creates drift issues for specs with validation errors.
Spec discovery and reads are rooted in retained filesystem capabilities and immutable snapshots so
path replacement cannot redirect issue inspection or `--create` validation.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_issues` | `root: &Path, format: OutputFormat, create: bool` | `()` | Verify issue references across all specs and optionally create drift issues |

## Invariants

1. Checks both `implements` and `tracks` frontmatter fields as explicit lists of numeric issue IDs;
   wrong field shapes or invalid entries make inspection inconclusive instead of becoming empty.
2. References from all specs are sent through one globally deduplicated, capped, and time-bounded
   GitHub verification batch.
3. Counts are tallied: valid (open), closed, not found (404), error (API failure)
4. Human-readable output prints no-reference guidance only when no spec references were gathered;
   all-error batches print a summary with the error count.
5. With `--create`, validates retained spec and mapped-source snapshots through the crate-private
   `validate_spec_content_with_sources` entry point and calls `create_drift_issues` for resulting
   validation errors
6. Exits 1 if any issue references are not found (404) or unverifiable
7. Specs are scanned before repository/provider resolution. An empty reference set performs no Git
   auto-detection or provider access; if `github.repo` is configured, its `owner/repository` syntax
   is still validated before no-reference or no-spec success, including when the configured specs
   directory is missing or empty.
8. Unreadable specs and malformed or missing frontmatter are retained as path-attributed,
   content-free inspection findings across every output format; they suppress no-reference
   guidance and make the command inconclusive with exit 1.
9. Recursive spec discovery is checked; traversal errors are findings rather than silently dropped
   entries. Display paths stay project-relative and content-free, sanitize terminal controls, and
   use valid escaped Markdown/GitHub table cells and code spans even for hostile filenames.
10. The project root, configured specs directory, recursive child directories, and spec files are
    opened and identity-checked through retained capabilities. Each discovered spec identity is
    compared before open, after open, throughout read completion, and against the verified handle;
    symlink, regular-file, and hardlink replacement cannot authorize replacement bytes.
11. Checked issue parsing rejects duplicate keys or malformed YAML anywhere and rejects
    blank/null/wrong-shaped top-level issue fields, while accepting comments/trailing commas and
    ignoring nested extension or block-scalar lookalikes.
12. Every renderer escapes control characters, bidirectional formatting controls, and Unicode line
    and paragraph separators (Zl/Zp); Markdown/GitHub additionally preserve one valid escaped table
    row and code span, padding the span content when a path begins or ends with a backtick. Safe
    relative paths use forward slashes on Windows while Unix preserves literal backslashes in
    filenames as data.
13. `--create` runs validation from immutable capability-rooted spec and mapped-source snapshots
    through `validate_spec_content_with_sources`; neither discovered spec paths nor mapped source
    paths are reopened for validation, and supplied-content TypeScript export extraction does not
    resolve wildcard imports through ambient paths.
14. Spec discovery retains no more than 10,000 snapshots, reads at most 4 MiB per spec, and retains
    at most 64 MiB of spec bytes cumulatively. Mapped-source snapshotting likewise limits each
    source observation to 4 MiB and retained source bytes to 64 MiB cumulatively.
15. A present selected project config is opened through the retained project capability, rejected
    when it is a symlink/reparse point or non-regular entry, identity-checked through the same
    handle, and read at most once into a 4 MiB snapshot. Parsing and all later configuration use
    those exact retained bytes; malformed UTF-8, JSON/TOML syntax, or known TOML field shapes are
    structured, content-free findings and cannot fall back to ambient/default paths.
16. Missing/empty specs and repository-resolution failures are rendered through the selected output
    format. JSON remains parseable, and Markdown/GitHub retain their structured headings and
    diagnostics instead of leaking an early text-only exit.

## Behavioral Examples

### Scenario: All references valid

- **Given** specs reference issues #10, #15, #20 — all exist and are open
- **When** `cmd_issues` runs
- **Then** prints "3 valid, 0 closed, 0 not found" and exits 0

### Scenario: Stale reference

- **Given** spec references issue #5 which was deleted
- **When** `cmd_issues` runs
- **Then** prints error for issue #5 and exits 1

## Error Cases

| Condition | Behavior |
|-----------|----------|
| GitHub repo unresolvable | Exits 1 with error message |
| No references and no configured repository | Prints no-reference guidance and exits 0 without Git auto-detection or provider access |
| No references with a configured repository | Validates `owner/repository` syntax, then prints no-reference guidance without Git auto-detection or provider access |
| No specs with a valid configured repository | Validates syntax, prints "No spec files found.", and returns without Git auto-detection or provider access |
| Invalid configured repository syntax, even with missing/empty specs | Exits 1 before no-spec/no-reference success |
| `GITHUB_TOKEN` unavailable or invalid | REST API calls fail, counted as errors; authenticated `gh` state is not consulted |
| Issue returns 404 | Counted as "not found", triggers non-zero exit |
| API rate limit | Counted as "error", reported but does not halt |
| Repository inaccessible or provider malformed/timed out | Counted as error, never not-found |
| More than 100 unique issue IDs | Batch error before provider access |
| Every referenced issue is unverifiable | Prints an error-count summary, never no-reference guidance, and exits 1 |
| Spec cannot be read or has malformed/missing frontmatter | Reports a content-free, path-attributed inspection finding in text/JSON/Markdown/GitHub output, never prints no-reference guidance, and exits 1 |
| `implements` or `tracks` has a scalar, mapping, mixed list, or non-numeric entry | Reports malformed frontmatter instead of silently discarding the bad shape or value |
| Duplicate key or malformed YAML elsewhere in frontmatter | Reports malformed frontmatter; complete checked parsing fails |
| Recursive spec discovery encounters an unreadable/disappearing entry | Reports an inconclusive traversal finding instead of treating the undiscovered subtree as empty |
| Spec filename contains controls, pipes, backticks, or newlines | Emits a sanitized project-relative path; text cannot inject terminal controls and Markdown/GitHub remains one valid table row with a valid code span |
| Spec filename contains bidi formatting controls or Unicode Zl/Zp separators | Emits escaped text in every renderer; visual order and row/line structure cannot be injected |
| A discovered spec path is replaced before or during reading | Same-handle identity validation rejects the entry or retains the original snapshot; replacement bytes are never trusted |
| A discovered spec is replaced with a regular file or hardlink before read | Identity mismatch becomes a safe read finding; replacement bytes are never parsed |
| A spec exceeds 4 MiB, discovery exceeds 10,000 specs, or retained spec bytes exceed 64 MiB | Reports bounded inspection findings and exits 1 rather than retaining an unbounded snapshot set |
| Retained mapped sources exceed per-file or cumulative limits | Source observations become unreadable/rejected validation inputs; validation never falls back to ambient reopening |
| Selected project config is unreadable, invalid UTF-8, or malformed JSON/TOML | Reports a structured project-configuration finding and exits 1 without scanning fallback paths or claiming no specs |
| Selected project config is a symlink/reparse point, non-regular entry, over 4 MiB, replaced during read, or has a wrong-shaped known TOML field | Reports the same content-free configuration finding; no target/replacement bytes or ambient fallback path are used |
| Missing/empty specs or repository resolution fails under JSON/Markdown/GitHub output | Renders one valid selected-format document before returning the trustworthy success or failure exit |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| commands | `build_schema_columns`, `create_drift_issues` |
| config | `load_config` |
| github | `resolve_repo`, GitHub API calls |
| parser | `parse_checked_issue_references`, `parse_frontmatter` |
| types | `OutputFormat` |
| validator | `SourceSnapshot`, `validate_spec_content_with_sources`, `get_schema_table_names`, `normalize_source_mapping`, `source_within_root` |
| ignore | `IgnoreRules` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync issues` subcommand |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-22 | CHG-0063: Batch, deduplicate, cap, fail closed, and report all-error GitHub verification truthfully |
| 2026-07-22 | CHG-0063: Skip repository and provider resolution when no issue references are present |
| 2026-07-22 | CHG-0063 independent-review follow-up: Fail closed on unreadable/malformed specs, invalid issue-field shapes, traversal errors, and hostile diagnostic paths |
| 2026-07-22 | CHG-0063 final adversarial follow-up: Use one retained project capability for bounded same-handle spec/source snapshots and `--create` validation, cap all recursive entries, reject regular/hardlink replacement, validate configured repo syntax even with missing/empty specs, pad edge-backtick code spans, and sanitize hostile renderer input |
| 2026-07-22 | CHG-0063 Windows CI follow-up: Normalize Windows diagnostic path separators to forward slashes while preserving literal Unix filename backslashes, and repair junction fixtures to use native path joins |
| 2026-07-22 | CHG-0063 adversarial follow-up: Fail closed when a selected project config is unreadable or malformed so configured specs cannot disappear behind default-path no-spec success |
| 2026-07-22 | CHG-0063 final configuration follow-up: Read selected config through one retained, bounded, same-handle snapshot; reject special entries and wrong-shaped TOML fields; preserve structured early output |
