## ADDED

### REQUIREMENT REQ-commands-002

Check reporting SHALL expose planned mapping notices separately from warnings in text, JSON, Markdown, and GitHub formats.

Acceptance Criteria

- `run_validation` returns deterministic notice strings as a seventh tuple member.
- Text output identifies each planned path without printing a misleading all-files-exist check.
- Structured JSON includes a deterministic notices array.
- Markdown and GitHub reports include a planned mappings section.
- Notice-only results remain passing under strict enforcement.


## MODIFIED

### SPEC SECTION Public API

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `load_and_discover` | `root: &Path, allow_empty: bool` | `(SpecSyncConfig, Vec<PathBuf>)` | Load config and discover all spec files (excluding `_`-prefixed); exits if empty and `allow_empty` is false |
| `validate_module_name` | `module_name: &str` | `Result<(), String>` | Validate a user-supplied module name for the scaffolding commands (`new`, `add-spec`, `scaffold`, `wizard`): must be a single plain path segment (one `Component::Normal`, no separators/`.`/`..`/absolute/drive-relative/control chars), preventing path traversal outside the project |
| `filter_specs` | `root: &Path, spec_files: &[PathBuf], filters: &[String]` | `Vec<PathBuf>` | Filter spec files by user-provided names/paths (exact path, relative path, filename, module name); returns all if filters is empty |
| `filter_by_status` | `spec_files: &[PathBuf], exclude: &[String], only: &[String]` | `Vec<PathBuf>` | Filter spec files by their frontmatter status field; supports exclude-list and allow-list modes |
| `build_schema_columns` | `root: &Path, config: &SpecSyncConfig` | `HashMap<String, SchemaTable>` | Build column-level schema from migration files if `schema_dir` is configured |
| `run_validation` | `root: &Path, spec_files: &[PathBuf], ownership_spec_files: &[PathBuf], schema_tables: &HashSet<String>, schema_columns: &HashMap<String, schema::SchemaTable>, config: &types::SpecSyncConfig, collect: bool, explain: bool, ignore_rules: &IgnoreRules` | `(usize, usize, usize, usize, Vec<String>, Vec<String>, Vec<String>)` | Run validation on all spec files returning (errors, warnings, passed, total, error_strings, warning_strings, notice_strings); contains full text rendering logic |
| `compute_exit_code` | `total_errors, total_warnings, strict, enforcement, coverage, require_coverage` | `i32` | Compute exit code without printing or exiting based on enforcement mode |
| `exit_with_status` | `total_errors, total_warnings, strict, enforcement, coverage, require_coverage` | `!` | Same as `compute_exit_code` but prints messages and calls `process::exit()` |
| `create_drift_issues` | `root, config, all_errors, format` | `()` | Create GitHub issues for specs with validation errors, grouping errors by spec path |

**Re-exported Submodules**

| Module | Description |
|--------|-------------|
| `agents` | Native AI-tool skill/slash-command installation dispatch (Claude Code, Cursor, Codex, Gemini CLI) |
| `archive_tasks` | Archive completed tasks from companion files |
| `changelog` | Generate spec changelog between git refs |
| `change` | Verified SDD change lifecycle command dispatch |
| `check` | Main validation command |
| `comment` | Post spec check summary as PR comment |
| `compact` | Compact changelog entries |
| `coverage` | File and LOC coverage reporting |
| `deps` | Dependency graph validation and visualization |
| `diff` | Show export drift since a git ref |
| `generate` | Scaffold specs for unspecced modules |
| `hooks` | Agent/IDE hook management |
| `import` | Import specs from GitHub/Jira/Confluence |
| `init` | Create the current `.specsync/` project layout, TOML config, and SDD policy |
| `init_registry` | Create specsync-registry.toml |
| `issues` | Verify GitHub issue references |
| `merge` | Auto-resolve merge conflicts in specs |
| `new` | Quick-create minimal specs |
| `report` | Per-module coverage report with staleness |
| `resolve` | Resolve cross-project dependency refs |
| `rules` | List active validation rules (built-in and custom) |
| `stale` | Git-based staleness detection for spec drift |
| `scaffold` | Full spec scaffolding with templates |
| `score` | Spec quality scoring (0-100, A-F) |
| `view` | Role-filtered spec rendering |
| `wizard` | Interactive spec creation wizard |
| `lifecycle` | Spec lifecycle status transitions (promote, demote, set, status) |
| `migrate` | v3.x to v4.0.0 project migration (config relocation, lifecycle extraction) |
| `rehash` | Regenerate hash cache for all specs |
