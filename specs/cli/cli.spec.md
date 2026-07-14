---
module: cli
version: 10
status: stable
files:
  - src/main.rs
db_tables: []
tracks: [120]
depends_on:
  - specs/agents/agents.spec.md
  - specs/archive/archive.spec.md
  - specs/change/change.spec.md
  - specs/changelog/changelog.spec.md
  - specs/cli_args/cli_args.spec.md
  - specs/cmd_change/cmd_change.spec.md
  - specs/commands/commands.spec.md
  - specs/comment/comment.spec.md
  - specs/compact/compact.spec.md
  - specs/config/config.spec.md
  - specs/deps/deps.spec.md
  - specs/exports/exports.spec.md
  - specs/generator/generator.spec.md
  - specs/git_utils/git_utils.spec.md
  - specs/github/github.spec.md
  - specs/hash_cache/hash_cache.spec.md
  - specs/hooks/hooks.spec.md
  - specs/ignore/ignore.spec.md
  - specs/importer/importer.spec.md
  - specs/manifest/manifest.spec.md
  - specs/mcp/mcp.spec.md
  - specs/merge/merge.spec.md
  - specs/output/output.spec.md
  - specs/parser/parser.spec.md
  - specs/registry/registry.spec.md
  - specs/schema/schema.spec.md
  - specs/scoring/scoring.spec.md
  - specs/types/types.spec.md
  - specs/util/util.spec.md
  - specs/validator/validator.spec.md
  - specs/view/view.spec.md
  - specs/watch/watch.spec.md
---

# CLI

## Purpose

The `specsync` command-line entry point parses global options, blocks configured verification children from recursively dispatching `check`, `change`, or `lifecycle` commands, routes canonical validation and verified SDD lifecycle commands to focused handlers, and preserves equivalent human-readable and structured output without owning domain policy.

## Public API

This module is the binary entry point (main.rs). All functions are private — there are no `pub` exports. The "API" is the CLI interface itself, documented below.

### CLI Structure

Clap derive types define the root `Cli`, the `Command` namespace, and focused action enums such as `HooksAction`, `AgentsAction`, `LifecycleAction`, and `ChangeAction`.

### Subcommands

| Command | Description | Key Flags |
|---------|-------------|-----------|
| check | Validate all specs against source code (default when no subcommand given) | --strict, --require-coverage N, --json, --fix, --force, --create-issues, --explain, [SPEC...] |
| coverage | Show file and module coverage report | --strict, --require-coverage N, --json |
| generate | Deterministically scaffold spec files for unspecced modules | --uncovered, --batch MODULE... |
| init | Create the 5.0 `.specsync/` layout, TOML config, SDD policy, and version stamp | — |
| change | Manage the verified SDD lifecycle, interviews, approvals, verification, acceptance, adoption, and archive | new, answer, approve, start, verify, accept, archive, adopt |
| score | Score spec quality (0–100) with letter grades and suggestions | --json, --explain, [SPEC...] |
| watch | Watch spec and source files, re-running check on changes | --strict, --require-coverage N |
| mcp | Run as an MCP (Model Context Protocol) server over stdio | — |
| add-spec | Scaffold a new spec with companion files (tasks.md, context.md) | name positional arg |
| scaffold | Full scaffold: spec + companions + source detection + registry entry | name, --dir PATH, --template PATH |
| init-registry | Generate a specsync-registry.toml for cross-project references | --name |
| resolve | Resolve cross-project spec references in depends_on | --remote (enables network fetches) |
| diff | Show export changes since a git ref (useful for CI/PR comments) | --base REF (default: HEAD), --json |
| hooks install | Install agent instructions and/or git hooks | --claude, --cursor, --copilot, --agents, --precommit, --claude-code-hook |
| hooks uninstall | Remove previously installed hooks | --claude, --cursor, --copilot, --agents, --precommit, --claude-code-hook |
| hooks status | Show installation status of all hooks | — |
| compact | Compact changelog tables by summarizing old entries | --keep N (default 10), --dry-run |
| archive-tasks | Move completed task items to archive section | --dry-run |
| view | Filter spec content by stakeholder role | --role (dev\|qa\|product\|agent), --spec PATH |
| merge | Auto-resolve git merge conflicts in spec files | --dry-run, --all, --json |
| issues | Verify GitHub issue references in spec frontmatter | --create (create drift issues for failures) |
| wizard | Interactive guided spec creation with prompts and preview | — |
| import | Import specs from external systems (GitHub Issues, Jira, Confluence) | SOURCE, ID, --repo |
| new | Quick-create a minimal spec with auto-detected source files | name, --full |
| deps | Validate cross-module dependency graph | --mermaid, --dot, --json |
| report | Per-module coverage report with stale and incomplete detection | --stale-threshold N (default 5) |
| comment | Post spec-check summary as a PR comment (or print for piping) | --pr N, --base REF (default: main) |
| changelog | Generate changelog of spec changes between two git refs | RANGE positional arg |

### Global Flags

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| --strict | bool | false | Treat warnings as errors (exit 1) |
| --require-coverage | Option usize | None | Fail if file coverage percent is below threshold |
| --root | Option PathBuf | cwd | Project root directory |
| --format | text\|json\|markdown | text | Output format: colored text, machine-readable JSON, or markdown |
| --json | bool | false | Shorthand for `--format json` |
| --enforcement | Option EnforcementMode | None | Override configured enforcement mode (warn, enforce-new, strict) |
| --force | bool | false | Bypass hash cache and re-validate all specs |

### Internal Functions

All functions in main.rs are private (no pub keyword). Key internal functions:

- **main** — Parse CLI args, canonicalize root, dispatch to subcommand handler
- **cmd_init** — Create the current `.specsync/` layout with auto-detected source dirs; no-op if config exists
- **cmd_change** — Dispatch the verified SDD lifecycle and render equivalent text/JSON results
- **cmd_check** — Load config, discover specs, validate, print results, exit with status
- **cmd_coverage** — Load config, compute coverage, print detailed coverage report
- **cmd_generate** — Deterministically scaffold specs for unspecced modules
- **cmd_score** — Score all specs and print quality grades
- **cmd_add_spec** — Create a single spec + companion files for a named module
- **cmd_init_registry** — Generate specsync-registry.toml from existing specs
- **cmd_resolve** — Resolve local and cross-project depends_on references
- **cmd_hooks** — Dispatch to hooks install/uninstall/status
- **cmd_diff** — Compare exports across git refs, show new/removed exports per spec
- **cmd_compact** — Compact changelog tables in all specs
- **cmd_archive_tasks** — Move completed tasks to archive section in companion files
- **cmd_view** — Filter and display spec content for a specific role
- **cmd_merge** — Auto-resolve git merge conflicts in spec files
- **cmd_issues** — Verify GitHub issue references in spec frontmatter
- **cmd_wizard** — Interactive wizard for guided spec creation with template selection and preview
- **cmd_import** — Import specs from external systems (GitHub Issues, Jira, Confluence) using `importer` module
- **cmd_report** — Per-module coverage report with stale detection (modules N+ commits behind source)
- **cmd_comment** — Post spec-check summary as a PR comment via `gh`, or print the comment body
- **cmd_changelog** — Generate a changelog of spec changes between two git refs
- **cmd_scaffold** — Full module scaffold: spec + companion files + source detection + registry entry
- **auto_fix_specs** — Scan source files for undocumented exports and auto-add skeleton rows to spec Public API tables
- **collect_hook_targets** — Convert boolean flags to Vec of HookTarget
- **load_and_discover** — Load config and find all spec files (filtering _-prefixed templates)
- **run_validation** — Validate all specs, return counts and collected error/warning strings
- **compute_exit_code** — Determine process exit code from errors, warnings, strict mode, and coverage
- **print_summary** — Print "N specs checked: X passed, Y warnings, Z failed"
- **print_coverage_line** — Print file and LOC coverage percentages with color coding
- **print_coverage_report** — Print detailed list of unspecced modules and files
- **exit_with_status** — Print messages and process::exit based on errors/warnings/coverage

## Invariants

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
24. Inherited verification context rejects `check`, `change`, and `lifecycle` before handler dispatch, while unrelated commands preserve their current behavior.

## Behavioral Examples

### Scenario: Default subcommand

- **Given** the user runs `specsync` with no subcommand
- **When** the CLI parses arguments
- **Then** the `check` command executes

### Scenario: Strict mode with warnings

- **Given** specs have undocumented exports (warnings but no errors)
- **When** `specsync check --strict` is run
- **Then** the process exits with code 1

### Scenario: JSON output

- **Given** `--json` flag is passed
- **When** any command runs
- **Then** output is valid JSON with no ANSI escape codes

### Scenario: Init idempotency

- **Given** a config (v4 `.specsync/config.toml` or legacy `specsync.json`) already exists
- **When** `specsync init` is run
- **Then** prints an "already exists" message and returns without modifying it

### Scenario: Coverage threshold

- **Given** file coverage is 80%
- **When** `specsync check --require-coverage 90` is run
- **Then** the process exits with code 1 and prints the unspecced files

### Scenario: Deterministic generation

- **Given** uncovered modules exist
- **When** `specsync generate` is run
- **Then** local template specs and companion files are generated

### Scenario: Resolve without network

- **Given** specs have cross-project `depends_on` refs
- **When** `specsync resolve` is run (without `--remote`)
- **Then** lists the refs but does not verify them against remote registries

### Scenario: Fix auto-adds undocumented exports

- **Given** a spec's source files have exports not documented in the Public API section
- **When** `specsync check --fix` is run
- **Then** skeleton rows for the missing exports are appended to the Public API section and the spec file is written to disk

### Scenario: Fix does not duplicate already-documented exports

- **Given** a spec already documents `login` but not `logout`
- **When** `specsync check --fix` is run
- **Then** only `logout` is added; `login` is not duplicated

### Scenario: Fix creates Public API section when missing

- **Given** a spec has no `## Public API` section
- **When** `specsync check --fix` is run
- **Then** a new `## Public API` section with a table header and skeleton export rows is appended to the spec

### Scenario: Diff shows new exports

- **Given** a source file has a new export added since the base ref
- **When** `specsync diff --base HEAD` is run
- **Then** the new export appears in `new_exports` for the affected spec

### Scenario: Diff shows removed exports

- **Given** a source file has an export removed since the base ref but the spec still documents it
- **When** `specsync diff --base HEAD` is run
- **Then** the removed export appears in `removed_exports` for the affected spec

### Scenario: Diff with no changes

- **Given** no source files have changed since the base ref
- **When** `specsync diff --base HEAD` is run
- **Then** output is empty (`{"changes":[]}` in JSON mode)

### Scenario: Hooks install with no flags

- **Given** no specific hook flags are passed
- **When** `specsync hooks install` is run
- **Then** `collect_hook_targets` returns empty vec, which hooks module interprets as "install all"

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Cannot determine cwd | Panics with "Cannot determine cwd" |
| Retired `--provider` or `--model` flag | Clap rejects the unknown argument |
| Failed to write the canonical config, SDD policy, or versioned layout during `init` | Prints an actionable error and exits 1 |
| Failed to create spec directory | Prints error to stderr and exits 1 |
| Failed to write spec file | Prints error to stderr and exits 1 |
| Failed to write `specsync-registry.toml` | Prints error to stderr and exits 1 |
| No spec files found (non-generate commands) | Prints guidance message and exits 0 |

## Performance Requirements

### Response Time Targets

| Operation | Target | Maximum |
|-----------|--------|---------|
| `check` (cached) | < 500ms | 2s |
| `check` (full validation) | < 2s | 5s |
| `coverage` | < 1s | 3s |
| `score` | < 1s | 3s |
| `generate` (local) | < 2s | 5s |
| `init` | < 500ms | 2s |
| `view` | < 200ms | 1s |
| `diff` | < 1s | 3s |
| `compact` | < 1s | 3s |
| `archive-tasks` | < 1s | 3s |
| `deps` | < 2s | 5s |
| `merge` | < 1s | 3s |
| `changelog` | < 2s | 5s |
| `comment` (local) | < 500ms | 2s |
| `report` | < 2s | 5s |

### Cache Requirements

| Cache Type | Invalidation Time | Behavior |
|------------|-------------------|----------|
| File hash cache | 5 seconds | `hash_cache` module updates cache entries within 5s of file changes |
| Spec parse cache | N/A | Parsed frontmatter is not cached; re-parsed on each run |
| Registry cache | 60 seconds | Remote registry entries cached for 60s with `--remote` flag |

### Resource Limits

| Resource | Limit | Behavior |
|----------|-------|----------|
| Memory | 512MB | CLI should not exceed 512MB heap for projects with < 100 specs |
| Concurrent file operations | 10 | Maximum 10 concurrent file reads during validation |
| HTTP timeout | 10s | GitHub API calls timeout after 10 seconds |
| Git operation timeout | 30s | Git diff/log operations timeout after 30 seconds |

### Scalability Targets

- Projects with up to **50 specs**: All operations complete within performance targets
- Projects with up to **100 specs**: `check` may exceed 2s but should complete within 5s
- Projects with **500+ specs**: Consider using `--force` to bypass cache only when necessary; incremental validation recommended

### Measurement

Performance is measured on a standard development machine:
- CPU: 4-core modern processor
- RAM: 16GB
- Storage: SSD
- Network: Standard broadband (for AI/network operations)

Cold start times (first run after boot) may be 2-3x higher due to disk cache warming.

## Dependencies

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

## Change Log

| Date | Change |
|------|--------|
| 2026-07-10 | v5: dispatch the verified `specsync change` SDD lifecycle |
| 2026-06-11 | v4: `--root` now errors (exit 2) for nonexistent paths; init scenario covers the v4 config layout |
| 2026-04-10 | Add Performance Requirements section with response time targets, cache requirements, resource limits, and scalability targets |
| 2026-03-25 | Initial spec |
| 2026-04-06 | Add compact, archive-tasks, view, merge, issues subcommands; add --force, --create-issues, --format flags; add hash_cache/github/archive/compact/view/merge dependencies |
| 2026-04-09 | Add scaffold, report, comment, changelog subcommands; add --enforcement and --explain flags; add --agents hook target; add comment/changelog/deps dependencies |
| 2026-07-11 | CHG-0003-finalize-specsync-5-0-release-consistency-and-parallel-validation: Finalize SpecSync 5.0 release consistency and parallel validation |
| 2026-07-11 | CHG-0007-harden-specsync-5-0-as-an-agent-native-secret-free-sdd-core-and-close-release-r: Harden SpecSync 5.0 as an agent-native, secret-free SDD core and close release regressions |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-14 | CHG-0025-address-all-unresolved-review-feedback-on-pr-366: Address all unresolved review feedback on PR 366 |
| 2026-07-14 | CHG-0029-address-all-remaining-review-feedback-from-pr-366: Address all remaining review feedback from PR 366 |
