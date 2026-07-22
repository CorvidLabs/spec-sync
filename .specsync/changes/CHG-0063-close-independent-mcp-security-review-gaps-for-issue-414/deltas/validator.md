## ADDED

### REQUIREMENT REQ-validator-008

Coverage gates SHALL use fallible checked manifest discovery and SHALL report malformed or unreadable
Gradle settings as inconclusive instead of accepting partial coverage.

Acceptance Criteria

- `compute_coverage_checked` propagates checked manifest-discovery errors without producing a
  partial `CoverageReport`.
- CLI and MCP coverage/enforcement callers use checked coverage and fail with an inconclusive
  diagnostic.
- `compute_coverage` remains available for compatibility and returns a zero-percent report carrying
  an inconclusive module diagnostic when checked discovery fails.

## MODIFIED

### SPEC SECTION Public API

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `validate_spec` | `spec_path: &Path, root: &Path, schema_tables: &HashSet<String>, schema_columns: &HashMap<String, SchemaTable>, config: &SpecSyncConfig` | `ValidationResult` | Validate a single spec file: frontmatter, files, sections, API surface, dependencies |
| `find_spec_files` | `dir: &Path` | `Vec<PathBuf>` | Recursively find all `*.spec.md` files in a directory |
| `compute_coverage` | `root, spec_files, config` | `CoverageReport` | Compatibility coverage wrapper that records checked discovery failures as an inconclusive zero-percent report |
| `compute_coverage_checked` | `root, spec_files, config` | `Result<CoverageReport, String>` | Compute coverage while surfacing malformed or unreadable manifest discovery as an inconclusive error for gate callers |
| `get_schema_table_names` | `root, config` | `HashSet<String>` | Extract table names from SQL schema files using configurable regex |
| `is_cross_project_ref` | `dep: &str` | `bool` | Check if a dependency string is a cross-project ref (`owner/repo@module`) |
| `parse_cross_project_ref` | `dep: &str` | `Option<(&str, &str)>` | Parse cross-project ref into (owner/repo, module) tuple |
| `normalize_source_mapping` | `file: &str` | `Option<String>` | Normalize a safe project-relative mapping by removing redundant current-directory segments and rejecting absolute, parent, or prefixed paths; callers also reject backslashes so ownership, validation, and coverage share one portable mapping contract |
| `source_within_root` | `root: &Path, file: &str` | `bool` | Whether a `files:` entry or nearest existing ancestor resolves inside the project root; dangling or unreadable ancestors fail closed, and absolute, parent, or symlink-parent escapes are rejected |

### SPEC SECTION Invariants

1. Validation is bidirectional: phantom documented exports are errors and undocumented code exports are warnings.
2. Missing frontmatter fields are errors, not warnings.
3. Cross-project refs are skipped during local validation and checked only by `specsync resolve`.
4. Coverage excludes test files and configured exclude patterns, including supported simplified glob forms without panicking on `**/**`.
5. Source discovery respects `source_extensions`; an empty list means all supported languages.
6. `find_spec_files` returns sorted results.
7. Schema extraction uses the configured `schema_pattern`.
8. Missing-file suggestions use Levenshtein distance with a maximum of three.
9. Flat source files are detected as modules except common entry points.
10. Sections without substantive content are reported as unfinished draft text rather than template markers.
11. `validate_spec` records parsed lifecycle status on `ValidationResult.status`, or `None` when frontmatter is unreadable.
12. Requirements companions are validated when present but remain optional for technical/internal modules under adaptive policy.
13. `compute_coverage_checked` propagates malformed or unreadable Gradle discovery instead of reporting partial coverage; the compatibility `compute_coverage` wrapper remains available, while CLI and MCP gates use the checked path.
