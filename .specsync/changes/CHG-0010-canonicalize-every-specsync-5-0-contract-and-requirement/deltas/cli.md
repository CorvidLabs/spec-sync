## MODIFIED

### SPEC SECTION Invariants

1. When no subcommand is given, `check` runs by default
2. `--root` defaults to the current working directory; the path is validated (must be an existing directory — otherwise an error is printed and the process exits 2) and canonicalized
3. `--strict` causes warnings to produce a non-zero exit code
4. `--require-coverage N` causes exit 1 if file coverage percent < N
5. `--json` switches all output to machine-readable JSON (no ANSI colors)
6. `cmd_init` is idempotent and never overwrites current or legacy project configuration
7. `cmd_init_registry` is idempotent — does nothing if `specsync-registry.toml` already exists
8. `cmd_add_spec` generates companion files even if the spec already exists
9. `cmd_generate` re-runs validation after generating new specs to include them in the summary
10. `cmd_resolve --remote` performs network calls; without the flag, cross-project refs are listed but not verified
11. `load_and_discover` filters out spec files starting with `_` (template files)
12. Exit codes: 0 = success, 1 = errors (or warnings in strict mode, or coverage below threshold)
13. `collect_hook_targets` with no flags set returns an empty vec, meaning "all targets"
14. `--fix` only adds exports not already documented in the spec (no duplicates)
15. `--fix` modifies spec files on disk — validation runs after fix so the fixed specs are re-checked
16. `--fix` with `--json` suppresses the human-readable fix summary but still writes the fix
17. `cmd_diff` shells out to `git diff --name-only <base>` to detect changed files
18. `cmd_diff` only reports specs whose `files:` frontmatter list intersects the changed file set
19. `cmd_scaffold` auto-detects source files, creates companion files, and registers the module in `specsync-registry.toml` if it exists
20. `cmd_report` flags modules whose specs are N+ commits behind their source files (default threshold: 5)
21. `cmd_comment` without `--pr` prints the comment body to stdout; with `--pr N` posts via `gh` CLI
22. `cmd_changelog` requires a git ref range (e.g., `v0.1..v0.2`); exits 1 if range is invalid
23. `--enforcement` CLI flag overrides the effective loaded configuration (`.specsync/config.toml` first, with legacy compatibility fallbacks); `--strict` implies strict enforcement

### SPEC SECTION Error Cases

| Condition | Behavior |
|-----------|----------|
| Cannot determine cwd | Panics with "Cannot determine cwd" |
| Retired `--provider` or `--model` flag | Clap rejects the unknown argument |
| Failed to write the canonical config, SDD policy, or versioned layout during `init` | Prints an actionable error and exits 1 |
| Failed to create spec directory | Prints error to stderr and exits 1 |
| Failed to write spec file | Prints error to stderr and exits 1 |
| Failed to write `specsync-registry.toml` | Prints error to stderr and exits 1 |
| No spec files found (non-generate commands) | Prints guidance message and exits 0 |

### SPEC SECTION Dependencies

**Consumes**

| Module | What is used |
|--------|-------------|
| config | `load_config`, `detect_source_dirs` |
| cli_args | Clap parser types, command/action enums, and global argument projection |
| parser | `parse_frontmatter` |
| validator | `validate_spec`, `find_spec_files`, `compute_coverage`, `get_schema_table_names`, `is_cross_project_ref`, `parse_cross_project_ref` |
| exports | `has_extension`, `get_exported_symbols` (used by auto_fix_specs and cmd_diff) |
| generator | `generate_specs_for_unspecced_modules`, `generate_specs_for_unspecced_modules_paths`, `generate_companion_files_for_spec` |
| scoring | `score_spec`, `compute_project_score`, `SpecScore` |
| registry | `generate_registry`, `fetch_remote_registry`, `RemoteRegistry` |
| mcp | `run_mcp_server` |
| watch | `run_watch` |
| hooks | `cmd_install`, `cmd_uninstall`, `cmd_status`, `HookTarget` |
| types | `SpecSyncConfig`, `CoverageReport` |
| archive | `archive_tasks` |
| compact | `compact_changelogs` |
| view | `view_spec`, `valid_roles` |
| github | `verify_spec_issues`, `create_drift_issue`, `resolve_repo` |
| hash_cache | `HashCache`, `classify_all_changes`, `update_cache` |
| merge | `merge_specs`, `print_results`, `results_to_json` |
| comment | `render_comment_body`, `detect_branch`, `SpecViolation` |
| changelog | `changelog_between_refs` |
| deps | `validate_deps`, `render_mermaid`, `render_dot` |

**Consumed By**

| Module | What is used |
|--------|-------------|
| (none) | `main.rs` is the top-level entry point — nothing imports it |

**Frontmatter Synchronization**

Implementation SHALL add these canonical dependency specs to `depends_on`: `specs/agents/agents.spec.md`,
`specs/cli_args/cli_args.spec.md`, `specs/commands/commands.spec.md`, `specs/git_utils/git_utils.spec.md`,
`specs/ignore/ignore.spec.md`, `specs/output/output.spec.md`, `specs/util/util.spec.md`. This YAML frontmatter
update is an explicit implementation edit because semantic section deltas do not apply frontmatter.
