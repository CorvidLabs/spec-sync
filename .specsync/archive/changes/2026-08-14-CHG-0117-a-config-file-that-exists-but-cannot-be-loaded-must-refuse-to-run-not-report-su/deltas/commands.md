## ADDED

### REQUIREMENT REQ-commands-009

No command SHALL report a verdict derived from configuration that failed to load.

Acceptance Criteria
- A command that reads specs refuses to run when the configuration records a load failure, and names the file.
- The refusal states how to proceed: fix the file, or remove it to use the built-in defaults deliberately.
- A project with a valid configuration, and a project with none, are both unaffected.
- The refusal is applied once at the shared entry point, so no command can omit it.

## MODIFIED

### SPEC SECTION Public API
**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `load_and_discover` | `root: &Path, allow_empty: bool` | `(SpecSyncConfig, Vec<PathBuf>)` | Load config and discover all spec files (excluding `_`-prefixed); exits if empty and `allow_empty` is false |
| `refuse_unloadable_config` | `config: &SpecSyncConfig` | `()` | Exit rather than report a verdict derived from a config file that exists but could not be loaded; applied once inside `load_and_discover` so no command can omit it |
| `validate_module_name` | `module_name: &str` | `Result<(), String>` | Validate a user-supplied module name as one portable path segment: reject traversal/control/Windows-invalid characters, trailing spaces/dots, Windows reserved device basenames (including before extensions), and names whose `<name>.spec.md` filename exceeds 255 UTF-8 bytes |
| `filter_specs` | `root: &Path, spec_files: &[PathBuf], filters: &[String]` | `Vec<PathBuf>` | Filter spec files by user-provided names/paths (exact path, relative path, filename, module name); returns all if filters is empty |
| `filter_by_status` | `spec_files: &[PathBuf], exclude: &[String], only: &[String]` | `Vec<PathBuf>` | Filter spec files by their frontmatter status field; supports exclude-list and allow-list modes |
| `build_schema_columns` | `root: &Path, config: &SpecSyncConfig` | `HashMap<String, SchemaTable>` | Compatibility column-map wrapper; checked command validation uses one fallible snapshot internally |
| `run_validation` | `root: &Path, spec_files: &[PathBuf], ownership_spec_files: &[PathBuf], config: &types::SpecSyncConfig, collect: bool, explain: bool, ignore_rules: &IgnoreRules` | `(usize, usize, usize, usize, Vec<String>, Vec<String>, Vec<String>)` | Validate all selected specs from one checked schema input and return error/warning/pass counts plus rendered diagnostics/notices |
| `compute_exit_code` | `total_errors, total_warnings, strict, enforcement, coverage, require_coverage` | `i32` | Compute exit code without printing or exiting based on enforcement mode |
| `exit_with_status` | `total_errors, total_warnings, strict, enforcement, coverage, require_coverage` | `!` | Same as `compute_exit_code` but prints messages and calls `process::exit()` |
| `create_drift_issues` | `root, config, all_errors, format` | `()` | Create GitHub issues for specs with validation errors, grouping errors by spec path; terminal diagnostics sanitize hostile repository, path, URL, and provider text before rendering, while the GitHub layer sanitizes issue title/body text |

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
| `spec_inventory` | List normalized inventory paths for selected specs |
| `MAX_MODULE_NAME_LEN` | Maximum scaffold module name length |
| `MODULE_NAME_RULES` | Scaffold naming rules text |
| `validate_scaffold_module_name` | Strict naming for newly scaffolded modules |
| `check_case_collision` | Reject case-only collisions with existing specs |
| `default_enforcement` | Resolve config enforcement when CLI omits override |

