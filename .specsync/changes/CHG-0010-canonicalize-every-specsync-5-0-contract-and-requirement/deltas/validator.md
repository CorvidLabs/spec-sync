## ADDED

### REQUIREMENT REQ-validator-001

The validator SHALL enforce bidirectional code-contract, metadata, dependency, schema, and coverage rules while accumulating actionable findings.

Acceptance Criteria
- Bidirectional validation: spec documents non-existent export = ERROR; code exports undocumented symbol = WARNING
- Missing frontmatter fields (module, version, status) produce errors, not warnings
- Cross-project refs (`owner/repo@module` format) are detected and skipped during local validation
- Coverage computation excludes test files and configured exclude patterns
- `find_spec_files` returns results sorted by path
- Schema validation uses configurable regex pattern via `schema_pattern` config
- File path suggestions use Levenshtein distance with max distance of 3
- Flat source files (not in subdirectories) are detected as modules, excluding common entry points (main.rs, lib.rs, mod.rs, index.ts, etc.)
- Source discovery respects `source_extensions` config
- Requirements companions are validated when present but remain optional for technical/internal modules under adaptive artifact policy.

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
| `source_within_root` | `root: &Path, file: &str` | `bool` | Whether a `files:` entry resolves inside the project root (rejects absolute/`..`/symlink escapes); shared guard for every export-extraction site |

### SPEC SECTION Dependencies

**Consumes**

| Module | What is used |
|--------|-------------|
| parser | `parse_frontmatter`, `get_spec_symbols`, `get_missing_sections` |
| exports | `get_exported_symbols`, `has_extension`, `is_test_file` |
| config | `default_schema_pattern` |
| types | `CoverageReport`, `ValidationResult`, `SpecSyncConfig` |

**Consumed By**

| Module | What is used |
|--------|-------------|
| main | `validate_spec`, `find_spec_files`, `compute_coverage`, `get_schema_table_names` |
| mcp | `validate_spec`, `find_spec_files`, `compute_coverage`, `get_schema_table_names` |
| archive | `find_spec_files` to locate spec companion files |
| compact | `find_spec_files` to locate all spec files |
| merge | `find_spec_files` to locate all spec files when `--all` is used |

**Frontmatter Synchronization**

Implementation SHALL add these canonical dependency specs to `depends_on`: `specs/util/util.spec.md`. This YAML frontmatter update is an explicit implementation edit because semantic section deltas do not apply frontmatter.
