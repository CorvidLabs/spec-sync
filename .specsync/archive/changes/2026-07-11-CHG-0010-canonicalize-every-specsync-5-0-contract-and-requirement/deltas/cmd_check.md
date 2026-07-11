## MODIFIED

### SPEC SECTION Public API

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_check` | `root: &Path, strict: bool, enforcement: Option<types::EnforcementMode>, require_coverage: Option<usize>, format: types::OutputFormat, fix: bool, dry_run: bool, backup: bool, force: bool, create_issues: bool, explain: bool, stale: Option<Option<usize>>, spec_filters: &[String], exclude_status: &[String], only_status: &[String]` | `()` | Main check command: load config, discover specs, optionally bypass cache, run validation, auto-fix if requested, format output, exit with appropriate code |

### SPEC SECTION Dependencies

**Consumes**

| Module | What is used |
|--------|-------------|
| commands | `load_and_discover`, `filter_specs`, `build_schema_columns`, `run_validation`, `compute_exit_code`, `exit_with_status`, `create_drift_issues` |
| hash_cache | `HashCache::load`, `save`, `is_changed` |
| ignore | `IgnoreRules::load` |
| output | `print_summary`, `print_coverage_line`, `print_check_markdown` |
| comment | `build_comment_body` |
| validator | `compute_coverage`, `validate_spec` |
| types | `SpecSyncConfig`, `OutputFormat`, `EnforcementMode`, `CoverageReport` |
| github | `resolve_repo` |

**Consumed By**

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync check` subcommand |

**Frontmatter Synchronization**

Implementation SHALL add these canonical dependency specs to `depends_on`: `specs/config/config.spec.md`, `specs/parser/parser.spec.md`, `specs/util/util.spec.md`. This YAML frontmatter update is an explicit implementation edit because semantic section deltas do not apply frontmatter.
