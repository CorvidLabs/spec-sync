---
module: validator
version: 1
status: stable
files:
  - src/validator.rs
db_tables: []
depends_on:
  - specs/types/types.spec.md
  - specs/parser/parser.spec.md
  - specs/exports/exports.spec.md
  - specs/config/config.spec.md
---

# Validator

## Purpose

Core validation engine for spec-sync. Validates individual spec files against source code (bidirectional), discovers spec and source files, extracts schema table names from SQL migrations, computes file and LOC coverage metrics, and resolves cross-project dependency references.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `validate_spec` | `spec_path, root, schema_tables, config` | `ValidationResult` | Validate a single spec file: frontmatter, files, sections, API surface, dependencies |
| `find_spec_files` | `dir: &Path` | `Vec<PathBuf>` | Recursively find all `*.spec.md` files in a directory |
| `compute_coverage` | `root, spec_files, config` | `CoverageReport` | Compute file and LOC coverage across all source directories |
| `get_schema_table_names` | `root, config` | `HashSet<String>` | Extract table names from SQL schema files using configurable regex |
| `is_cross_project_ref` | `dep: &str` | `bool` | Check if a dependency string is a cross-project ref (`owner/repo@module`) |
| `parse_cross_project_ref` | `dep: &str` | `Option<(&str, &str)>` | Parse cross-project ref into (owner/repo, module) tuple |

## Invariants

1. Validation is bidirectional: spec documenting non-existent exports = ERROR; code exports not in spec = WARNING
2. Missing frontmatter fields (module, version, status, files) are errors, not warnings
3. Cross-project refs (`owner/repo@module`) are skipped during local validation — only checked by `specsync resolve`
4. Coverage computation excludes test files and configured exclude patterns
5. Source file discovery respects `source_extensions` config — empty means all supported languages
6. `find_spec_files` returns sorted results
7. Schema table extraction supports configurable regex patterns via `schema_pattern` config
8. File suggestions use Levenshtein distance (max 3) when a referenced source file is missing
9. Flat source files (e.g. `src/config.rs`) are detected as modules, excluding common entry points (main, lib, mod, index, app, `__init__`)

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

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Spec file unreadable | Error: "Cannot read spec" |
| Missing frontmatter delimiters | Error: "Missing or malformed YAML frontmatter" |
| Source file not found | Error with fix suggestion (Levenshtein-based or removal) |
| DB table not in schema | Error: "DB table not found in schema" |
| Missing required section | Error: "Missing required section: ## SectionName" |
| Dependency spec not found | Error: "Dependency spec not found" |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| parser | `parse_frontmatter`, `get_spec_symbols`, `get_missing_sections` |
| exports | `get_exported_symbols`, `has_extension`, `is_test_file` |
| config | `default_schema_pattern` |
| types | `CoverageReport`, `ValidationResult`, `SpecSyncConfig` |

### Consumed By

| Module | What is used |
|--------|-------------|
| main | `validate_spec`, `find_spec_files`, `compute_coverage`, `get_schema_table_names` |
| mcp | `validate_spec`, `find_spec_files`, `compute_coverage`, `get_schema_table_names` |

## Change Log

| Date | Change |
|------|--------|
| 2026-03-25 | Initial spec |
