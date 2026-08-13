---
module: cmd_issues
version: 12
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

1. The command gathers all `implements` and `tracks` references before repository/provider access.
2. Project-wide verification is globally deduplicated, capped, and time-bounded by the GitHub
   module.
3. Inconclusive provider outcomes remain errors and cannot become successful not_found results.
4. No-reference guidance is emitted only when no spec references were gathered.
5. An empty reference set performs no Git auto-detection or provider access; configured repository
   syntax is still validated even when no specs were discovered.
6. A scan is empty only when every discovered spec was read and parsed successfully; unreadable or
   malformed specs are retained as safe findings and make verification inconclusive.
7. Recursive discovery and reads remain capability-rooted, and parsing consumes immutable bytes
   whose discovered identity remains binding through read, including regular/hardlink replacement.
8. Every output renderer escapes control, bidi, and Unicode line/paragraph separator characters.
9. Markdown/GitHub code spans pad content when a path begins or ends with a backtick.
10. Discovery retains at most 10,000 specs, at most 4 MiB per spec, and at most 64 MiB
    cumulatively; mapped-source retention applies a 4 MiB per-file and 64 MiB cumulative ceiling.
11. `--create` validates retained spec/source snapshots through
    `validate_spec_content_with_sources` and never reopens discovered paths or ambient wildcard
    targets.
12. Spec and mapped-source reads derive from one retained project capability, and recursive
    discovery examines no more than 100,000 total entries including non-spec entries.
13. Selected config is retained, same-handle identity-checked, bounded to 4 MiB, and parsed from
    exact bytes; malformed, wrong-shaped, linked, non-regular, replaced, or oversized input cannot
    produce fallback no-spec/no-reference success.
14. Finding paths normalize separators only on Windows; Unix literal backslashes remain data.
15. Missing/empty specs and repository-resolution failures use the selected structured renderer.
16. Omitted source directories are detected through a bounded sparse snapshot rooted in the
    retained project capability, never through a replaceable ambient root pathname.
17. Retained discovery skips shared ignored names before metadata inspection and never silently
    omits a recognized non-regular manifest.
18. Selected config and recognized manifests use no-follow, non-blocking retained handles;
    regular-file replacement and FIFO substitution between discovery and read are structured
    inconclusive findings on Windows and Unix.

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
| Omitted source directories with an ignored-name symlink | Skips the ignored name without following or rejecting its target |
| Omitted source directories with a FIFO/device at a recognized manifest name | Exits 1 with a structured configuration finding without blocking |
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
| 2026-07-22 | CHG-0063 capability-source follow-up: Detect omitted source directories through a bounded retained-capability snapshot rather than a replaceable ambient root |
| 2026-07-22 | CHG-0063 final agent-review follow-up: align retained ignored-directory behavior and reject special-file manifests as inconclusive |
| 2026-07-23 | CHG-0063 retained-handle follow-up: use no-follow, non-blocking config/manifest acquisition and reject regular-file replacement or FIFO substitution between discovery and read |
| 2026-07-27 | CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414: Close independent MCP security review gaps for issue 414 |
| 2026-08-13 | CHG-0107-fix-the-first-five-minutes-of-spec-sync-init-leaves-a-repo-that-fails-check-sc: Fix the first five minutes of spec-sync: init leaves a repo that fails check, scaffold writes prose that check rejects, and a directory in files: makes check silently green |
