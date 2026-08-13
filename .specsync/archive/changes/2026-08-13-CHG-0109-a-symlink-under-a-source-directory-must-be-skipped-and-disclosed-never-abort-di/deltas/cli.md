## MODIFIED

### SPEC SECTION Dependencies

**Consumes**

| Module | What is used |
|--------|-------------|
| config | `load_config`, `detect_source_dirs` |
| cli_args | Clap parser types, command/action enums, and global argument projection |
| parser | `parse_frontmatter` |
| validator | `validate_spec`, `find_spec_files`, `compute_coverage` (whose report carries the symlinked entries discovery skipped), `get_schema_table_names`, `is_cross_project_ref`, `parse_cross_project_ref` |
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

