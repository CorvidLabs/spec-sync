## ADDED

### REQUIREMENT REQ-validator-007

Validation SHALL treat safe normalized missing draft file mappings as planned by default without adding nonexistent files to current coverage.

Acceptance Criteria

- Draft planned mappings pass strict validation with explicit notices.
- Activating the spec or enabling `require_draft_files` restores the missing-file error.
- Creating the file transitions it to normal mapping and coverage.
- Existing files retain containment, readability, and duplicate-ownership validation.
- Incremental checks detect owners from unchanged cached specs.
- Redundant dot segments cannot create coverage mismatches.
- Unsafe paths remain errors in every lifecycle status.


## MODIFIED

### SPEC SECTION Public API

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `validate_spec` | `spec_path: &Path, root: &Path, schema_tables: &HashSet<String>, schema_columns: &HashMap<String, SchemaTable>, config: &SpecSyncConfig` | `ValidationResult` | Validate a single spec file: frontmatter, files, sections, API surface, dependencies |
| `find_spec_files` | `dir: &Path` | `Vec<PathBuf>` | Recursively find all `*.spec.md` files in a directory |
| `compute_coverage` | `root, spec_files, config` | `CoverageReport` | Compute file and LOC coverage across all source directories |
| `get_schema_table_names` | `root, config` | `HashSet<String>` | Extract table names from SQL schema files using configurable regex |
| `is_cross_project_ref` | `dep: &str` | `bool` | Check if a dependency string is a cross-project ref (`owner/repo@module`) |
| `parse_cross_project_ref` | `dep: &str` | `Option<(&str, &str)>` | Parse cross-project ref into (owner/repo, module) tuple |
| `normalize_source_mapping` | `file: &str` | `Option<String>` | Normalize a safe project-relative mapping by removing redundant current-directory segments and rejecting absolute, parent, or prefixed paths; shared by ownership and coverage |
| `source_within_root` | `root: &Path, file: &str` | `bool` | Whether a `files:` entry resolves inside the project root (rejects absolute/`..`/symlink escapes); shared guard for every export-extraction site |
