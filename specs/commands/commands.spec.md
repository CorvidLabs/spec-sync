---
module: commands
version: 1
status: stable
files:
  - src/commands/mod.rs
db_tables: []
tracks: []
depends_on:
  - specs/config/config.spec.md
  - specs/ignore/ignore.spec.md
  - specs/schema/schema.spec.md
  - specs/scoring/scoring.spec.md
  - specs/types/types.spec.md
  - specs/validator/validator.spec.md
  - specs/github/github.spec.md
---

# Commands

## Purpose

Shared command infrastructure used by all CLI subcommands. Provides config loading, spec discovery, spec filtering, schema column building, the central validation pipeline, exit code computation, and GitHub drift issue creation. Every command module imports from here rather than duplicating this boilerplate.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `load_and_discover` | `root: &Path, allow_empty: bool` | `(SpecSyncConfig, Vec<PathBuf>)` | Load config and discover all spec files (excluding `_`-prefixed); exits if empty and `allow_empty` is false |
| `filter_specs` | `root: &Path, spec_files: &[PathBuf], filters: &[String]` | `Vec<PathBuf>` | Filter spec files by user-provided names/paths (exact path, relative path, filename, module name); returns all if filters is empty |
| `build_schema_columns` | `root: &Path, config: &SpecSyncConfig` | `HashMap<String, SchemaTable>` | Build column-level schema from migration files if `schema_dir` is configured |
| `run_validation` | `root, spec_files, schema_tables, schema_columns, config, collect, explain, ignore_rules` | `(usize, usize, usize, usize, Vec<String>, Vec<String>)` | Run validation on all spec files returning (errors, warnings, passed, total, error_strings, warning_strings); contains full text rendering logic |
| `compute_exit_code` | `total_errors, total_warnings, strict, enforcement, coverage, require_coverage` | `i32` | Compute exit code without printing or exiting based on enforcement mode |
| `exit_with_status` | `total_errors, total_warnings, strict, enforcement, coverage, require_coverage` | `!` | Same as `compute_exit_code` but prints messages and calls `process::exit()` |
| `create_drift_issues` | `root, config, all_errors, format` | `()` | Create GitHub issues for specs with validation errors, grouping errors by spec path |

### Re-exported Submodules

| Module | Description |
|--------|-------------|
| `archive_tasks` | Archive completed tasks from companion files |
| `changelog` | Generate spec changelog between git refs |
| `check` | Main validation command |
| `comment` | Post spec check summary as PR comment |
| `compact` | Compact changelog entries |
| `coverage` | File and LOC coverage reporting |
| `deps` | Dependency graph validation and visualization |
| `diff` | Show export drift since a git ref |
| `generate` | Scaffold specs for unspecced modules |
| `hooks` | Agent/IDE hook management |
| `import` | Import specs from GitHub/Jira/Confluence |
| `init` | Create specsync.json config |
| `init_registry` | Create specsync-registry.toml |
| `issues` | Verify GitHub issue references |
| `merge` | Auto-resolve merge conflicts in specs |
| `new` | Quick-create minimal specs |
| `report` | Per-module coverage report with staleness |
| `resolve` | Resolve cross-project dependency refs |
| `rules` | List active validation rules (built-in and custom) |
| `scaffold` | Full spec scaffolding with templates |
| `score` | Spec quality scoring (0-100, A-F) |
| `view` | Role-filtered spec rendering |
| `wizard` | Interactive spec creation wizard |

## Invariants

1. `load_and_discover` excludes spec files starting with `_` (underscore prefix marks internal/template specs)
2. `filter_specs` matches against four forms: exact path, relative path, filename stem, and module name (stem minus `.spec` suffix)
3. `run_validation` applies ignore rules (global, inline, per-spec) to filter warnings before counting
4. Exit code logic by enforcement mode: Warn → always 0; EnforceNew → 1 if unspecced files; Strict → 1 on errors, also 1 on warnings when `--strict`
5. `--require-coverage N` triggers exit 1 if file coverage percent < N regardless of enforcement mode
6. `create_drift_issues` groups errors by spec path and creates one GitHub issue per spec, not per error

## Behavioral Examples

### Scenario: Filter by module name

- **Given** specs exist at `specs/auth/auth.spec.md` and `specs/api/api.spec.md`
- **When** `filter_specs(root, specs, &["auth"])` is called
- **Then** returns only `specs/auth/auth.spec.md`

### Scenario: Strict mode with warnings

- **Given** enforcement is `Strict`, `--strict` is set, validation has 0 errors but 3 warnings
- **When** `compute_exit_code()` is called
- **Then** returns 1 (warnings treated as errors)

### Scenario: EnforceNew with unspecced files

- **Given** enforcement is `EnforceNew`, coverage shows 2 unspecced files
- **When** `exit_with_status()` is called
- **Then** prints count and exits with code 1

## Error Cases

| Condition | Behavior |
|-----------|----------|
| No spec files found and `allow_empty` is false | Prints suggestion to run `specsync generate` and exits 0 |
| Filter matches no specs | Prints warning listing unmatched filters, returns empty vec |
| `schema_dir` not configured | `build_schema_columns` returns empty map (no error) |
| GitHub repo unresolvable for drift issues | Prints error and returns without creating issues |
| `gh` CLI fails to create issue | Prints per-spec error but continues with remaining specs |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| config | `load_config` |
| ignore | `IgnoreRules`, `parse_inline` |
| schema | `SchemaTable`, `build_schema` |
| scoring | `score_spec` (when explain mode) |
| types | `SpecSyncConfig`, `CoverageReport`, `EnforcementMode`, `OutputFormat` |
| validator | `find_spec_files`, `validate_spec` |
| github | `resolve_repo`, `create_drift_issue` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cmd_check | `load_and_discover`, `filter_specs`, `build_schema_columns`, `run_validation`, `compute_exit_code`, `exit_with_status`, `create_drift_issues` |
| cmd_coverage | `load_and_discover`, `build_schema_columns`, `run_validation`, `exit_with_status` |
| cmd_generate | `load_and_discover`, `build_schema_columns`, `run_validation`, `exit_with_status` |
| cmd_comment | `load_and_discover`, `build_schema_columns` |
| cmd_issues | `build_schema_columns`, `run_validation`, `create_drift_issues` |
| cmd_score | `load_and_discover`, `filter_specs` |
| cmd_report | `load_and_discover` |
| cmd_resolve | `load_and_discover` |
| cmd_diff | `load_and_discover` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
