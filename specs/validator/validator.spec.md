---
module: validator
version: 13
status: stable
files:
  - src/validator.rs
db_tables: []
tracks: [119]
depends_on:
  - specs/config/config.spec.md
  - specs/exports/exports.spec.md
  - specs/parser/parser.spec.md
  - specs/schema/schema.spec.md
  - specs/types/types.spec.md
  - specs/util/util.spec.md
---

# Validator

## Purpose

Core validation engine for spec-sync. Validates individual specs and selected companion artifacts against source code, discovers configured and zero-config source files including static HTML, HTM, and CSS content, rejects every known generated companion marker outside fenced examples, extracts schema table names from SQL migrations, computes non-vacuous file and LOC coverage metrics, preserves malformed manifest discovery as an inconclusive checked error for gates, and resolves cross-project dependency references.

## Public API

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `validate_spec` | `spec_path: &Path, root: &Path, schema_tables: &HashSet<String>, schema_columns: &HashMap<String, SchemaTable>, config: &SpecSyncConfig` | `ValidationResult` | Validate a single spec file: frontmatter, files, sections, API surface, dependencies |
| `validate_spec_content` | `spec_path: &Path, content: &str, root: &Path, schema_tables: &HashSet<String>, schema_columns: &HashMap<String, SchemaTable>, config: &SpecSyncConfig` | `ValidationResult` | Validate already-read spec bytes without reopening the spec or adjacent companions; mapped source files retain normal path-based validation behavior |
| `find_spec_files` | `dir: &Path` | `Vec<PathBuf>` | Recursively find all `*.spec.md` files in a directory |
| `compute_coverage` | `root, spec_files, config` | `CoverageReport` | Compute file and LOC coverage across all source directories |
| `compute_coverage_checked` | `root, spec_files, config` | `Result<CoverageReport, String>` | Compute coverage while surfacing malformed or unreadable manifest discovery as an inconclusive error for gate callers |
| `get_schema_table_names` | `root, config` | `HashSet<String>` | Extract table names from SQL schema files using configurable regex |
| `is_cross_project_ref` | `dep: &str` | `bool` | Check if a dependency string is a cross-project ref (`owner/repo@module`) |
| `parse_cross_project_ref` | `dep: &str` | `Option<(&str, &str)>` | Parse cross-project ref into (owner/repo, module) tuple |
| `normalize_source_mapping` | `file: &str` | `Option<String>` | Normalize a safe project-relative mapping by removing redundant current-directory segments and rejecting absolute, parent, or prefixed paths; callers also reject backslashes so ownership, validation, and coverage share one portable mapping contract |
| `source_within_root` | `root: &Path, file: &str` | `bool` | Whether a `files:` entry or the nearest existing filesystem ancestor of a missing leaf resolves inside the project root; dangling or unreadable ancestors fail closed, and absolute/`..`/symlink-parent escapes are rejected |

**Crate-Private Source-Snapshot API**

| Item | Parameters / Variants | Returns | Description |
|------|------------------------|---------|-------------|
| `SourceSnapshot` | `Present(Vec<u8>)`, `Missing`, `Rejected`, `Unreadable` | — | Capability-confined observation of a mapped source used by snapshot callers |
| `validate_spec_content_with_sources` | `spec_path, content, root, schema_tables, schema_columns, config, sources: &HashMap<String, SourceSnapshot>` | `ValidationResult` | Validate supplied spec bytes and supplied mapped-source observations without reopening either through ambient project paths |

## Invariants

1. Validation is bidirectional: spec documenting non-existent exports = ERROR; code exports not in spec = WARNING
2. Missing frontmatter fields (module, version, status, files) are errors, not warnings
3. Cross-project refs (`owner/repo@module`) are skipped during local validation — only checked by `specsync resolve`
4. Coverage computation excludes test files and configured exclude patterns. Exclude globs support `**/dir/**` (path contains `dir`), `**/*.ext` (suffix), and `**/name` (filename); a degenerate `**/**` matches every path (empty middle) and is handled without panicking
5. Source file discovery respects `source_extensions` config — empty means all supported languages
6. `find_spec_files` returns sorted results
7. Schema table extraction supports configurable regex patterns via `schema_pattern` config
8. File suggestions use Levenshtein distance (max 3) when a referenced source file is missing
9. Flat source files (e.g. `src/config.rs`) are detected as modules, excluding common entry points (main, lib, mod, index, app, `__init__`)
10. Sections with no substantive content are reported as unfinished draft text rather than as template markers
11. `validate_spec` records the spec's parsed lifecycle status on `ValidationResult.status` (None when frontmatter is unreadable) so reporters can surface status-based skips, e.g. drafts skipping section and export checks
12. Requirements companions are validated when present but optional for technical/internal modules under the adaptive 5.0 artifact model
13. `compute_coverage_checked` propagates malformed or unreadable Gradle discovery instead of reporting partial coverage; the compatibility `compute_coverage` wrapper remains available, while CLI and MCP gates use the checked path
14. `validate_spec_content` validates the caller-provided bytes, including CRLF normalization and
    size policy, without reopening `spec_path`; the path remains the logical location for
    diagnostics and mapped-source resolution, while adjacent companion checks are skipped.
15. `validate_spec` preserves the public path-based behavior by reading once and delegating the
    exact bytes to `validate_spec_content`.
16. `validate_spec_content_with_sources` is the crate-private exact spec-and-source snapshot entry
    point. Its supplied `SourceSnapshot` map is authoritative for mapped-source existence,
    readability, containment rejection, UTF-8 validation, and export extraction; it does not fall
    back to ambient source-path reads.

## Behavioral Examples

### Scenario: Valid spec passes

- **Given** a spec with correct frontmatter, all required sections, and API table matching code exports
- **When** `validate_spec` is called
- **Then** returns `ValidationResult` with empty errors and warnings

### Scenario: Spec documents non-existent export

- **Given** a spec listing `` `nonExistent` `` in the Public API table
- **When** `validate_spec` is called
- **Then** errors include "Spec documents 'nonExistent' but no matching export found in source"

### Scenario: Undocumented code export

- **Given** source code exports `helperFn` but the spec does not list it
- **When** `validate_spec` is called
- **Then** warnings include "Export 'helperFn' not in spec (undocumented)"

### Scenario: Cross-project dependency reference

- **Given** a spec with `depends_on: ["corvid-labs/algochat@auth"]`
- **When** `validate_spec` is called locally
- **Then** the cross-project ref is skipped (no error or warning)

### Scenario: Malformed Gradle settings make coverage inconclusive

- **Given** a configured source tree and malformed `settings.gradle` or `settings.gradle.kts`
- **When** `compute_coverage_checked` is called by a CLI or MCP gate
- **Then** it returns an error and the caller reports coverage as inconclusive instead of accepting partial totals

### Scenario: Validate a retained spec snapshot

- **Given** a caller already read spec bytes through a confined capability and the ambient
  `spec_path` is later replaced
- **When** `validate_spec_content(spec_path, content, ...)` is called
- **Then** validation uses only `content` for the spec and does not reopen the replaced path or
  adjacent companion files; mapped sources retain the normal path-based behavior

### Scenario: Validate retained spec and source snapshots

- **Given** a caller retained spec bytes and mapped-source observations, then the ambient spec and
  source paths are replaced
- **When** `validate_spec_content_with_sources(spec_path, content, ..., sources)` is called
- **Then** validation uses only the supplied spec bytes and `SourceSnapshot` map, never reopens
  either path, and extracts exports from supplied source content without ambient wildcard imports

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Spec file unreadable | Error: "Cannot read spec" |
| Pre-read snapshot passed to `validate_spec_content` | Validates those exact bytes without opening `spec_path` or adjacent companions |
| Supplied source is `Missing`, `Rejected`, or `Unreadable` | Reports the corresponding mapped-source validation outcome without ambient fallback |
| Missing frontmatter delimiters | Error: "Missing or malformed YAML frontmatter" |
| Source file not found | Error with fix suggestion (Levenshtein-based or removal) |
| DB table not in schema | Error: "DB table not found in schema" |
| Missing required section | Error: "Missing required section: ## SectionName" |
| Dependency spec not found | Error: "Dependency spec not found" |
| Malformed or unreadable Gradle settings during checked coverage | Returns `Err`; CLI/MCP gate callers report an inconclusive failure rather than coverage success |

## Dependencies

**Consumes**

| Module | What is used |
|--------|-------------|
| parser | `parse_frontmatter`, `get_spec_symbols`, `get_missing_sections` |
| exports | `get_exported_symbols`, `get_exported_symbols_from_content`, `has_extension`, `is_test_file` |
| config | `default_schema_pattern`, `discover_manifest_modules_checked` |
| types | `CoverageReport`, `ValidationResult`, `SpecSyncConfig` |

**Consumed By**

| Module | What is used |
|--------|-------------|
| main | `validate_spec`, `validate_spec_content`, `SourceSnapshot`, `validate_spec_content_with_sources`, `find_spec_files`, `compute_coverage_checked`, `get_schema_table_names` |
| mcp | `validate_spec`, `find_spec_files`, `compute_coverage_checked`, `get_schema_table_names` |
| archive | `find_spec_files` to locate spec companion files |
| compact | `find_spec_files` to locate all spec files |
| merge | `find_spec_files` to locate all spec files when `--all` is used |

**Frontmatter Synchronization**

Implementation SHALL add these canonical dependency specs to `depends_on`: `specs/util/util.spec.md`. This YAML frontmatter update is an explicit implementation edit because semantic section deltas do not apply frontmatter.

## Change Log

| Date | Change |
|------|--------|
| 2026-07-22 | v12: add checked coverage so malformed Gradle discovery fails CLI and MCP gates as inconclusive while retaining the compatibility report wrapper |
| 2026-07-22 | v13 / CHG-0063: add `validate_spec_content` so capability-rooted callers validate exact pre-read snapshots without reopening spec paths |
| 2026-07-10 | v5: keep coverage regression fixtures warning-free under current stable Clippy and document the intentionally in-file test-module layout |
| 2026-07-10 | v5: make canonical requirements companions adaptive rather than empty mandatory ceremony |
| 2026-07-02 | v4: add `source_within_root` — shared guard rejecting `files:` paths that escape the project root (absolute/`..`/symlink); applied in `validate_spec` and every export-extraction site (score, check --fix, diff, new) to close an out-of-root identifier-disclosure vector |
| 2026-06-11 | v3: `validate_spec` populates `ValidationResult.status` with the parsed lifecycle status so callers can report draft skips |
| 2026-06-07 | Update draft-only section warning wording |
| 2026-03-25 | Initial spec |
| 2026-04-06 | Document archive, compact, merge as consumers of find_spec_files; note hash_cache integration for incremental validation |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-14 | CHG-0024-stabilize-specsync-5-lifecycle-integrity-and-strict-validation-for-5-0-2: Stabilize SpecSync 5 lifecycle integrity and strict validation for 5.0.2 |
| 2026-07-14 | CHG-0025-address-all-unresolved-review-feedback-on-pr-366: Address all unresolved review feedback on PR 366 |
| 2026-07-14 | CHG-0034-support-extensionless-source-discovery-through-an-explicit-include-extensionless: Support extensionless source discovery through an explicit include_extensionless setting while preserving omitted and empty source_extensions defaults, with parser, scanner, strict file coverage, LOC coverage, and wizard regressions for extensionless-only and mixed projects |
| 2026-07-14 | CHG-0035-count-mjs-and-cjs-files-as-default-typescript-sources-so-mapped-and-uncovered-mo: Count mjs and cjs files as default TypeScript sources so mapped and uncovered module files contribute to strict file and LOC coverage denominators |
| 2026-07-14 | CHG-0039-allow-draft-specs-to-declare-planned-missing-source-mappings-without-failing-str: Allow draft specs to declare planned missing source mappings without failing strict validation while preserving path safety ownership enforcement exact coverage and complete notice contracts |
