---
module: cli
version: 1
status: stable
files:
  - src/main.rs
db_tables: []
tracks: [120]
depends_on:
  - specs/config/config.spec.md
  - specs/parser/parser.spec.md
  - specs/validator/validator.spec.md
  - specs/exports/exports.spec.md
  - specs/generator/generator.spec.md
  - specs/ai/ai.spec.md
  - specs/scoring/scoring.spec.md
  - specs/registry/registry.spec.md
  - specs/mcp/mcp.spec.md
  - specs/watch/watch.spec.md
  - specs/hooks/hooks.spec.md
  - specs/types/types.spec.md
  - specs/archive/archive.spec.md
  - specs/compact/compact.spec.md
  - specs/view/view.spec.md
  - specs/github/github.spec.md
  - specs/hash_cache/hash_cache.spec.md
  - specs/merge/merge.spec.md
---

# CLI

## Purpose

The `specsync` command-line interface — the main entry point for all user-facing operations. Parses CLI arguments via `clap`, routes to the appropriate subcommand handler, and orchestrates output formatting (colored text or JSON). Delegates all domain logic to the library modules; main.rs itself is purely a command dispatcher and output formatter.

## Public API

This module is the binary entry point (main.rs). All functions are private — there are no `pub` exports. The "API" is the CLI interface itself, documented below.

### CLI Structure

Three Clap derive structs define the CLI: Cli (root parser with global flags), Command (subcommand enum), and HooksAction (hooks sub-subcommand enum).

### Subcommands

| Command | Description | Key Flags |
|---------|-------------|-----------|
| check | Validate all specs against source code (default when no subcommand given) | --strict, --require-coverage N, --json, --fix, --force, --create-issues |
| coverage | Show file and module coverage report | --strict, --require-coverage N, --json |
| generate | Scaffold spec files for unspecced modules | --provider PROVIDER (AI mode: auto/claude/anthropic/openai/ollama/copilot) |
| init | Create a specsync.json config file with auto-detected source dirs | — |
| score | Score spec quality (0–100) with letter grades and suggestions | --json |
| watch | Watch spec and source files, re-running check on changes | --strict, --require-coverage N |
| mcp | Run as an MCP (Model Context Protocol) server over stdio | — |
| add-spec | Scaffold a new spec with companion files (tasks.md, context.md) | name positional arg |
| init-registry | Generate a specsync-registry.toml for cross-project references | --name |
| resolve | Resolve cross-project spec references in depends_on | --remote (enables network fetches) |
| diff | Show export changes since a git ref (useful for CI/PR comments) | --base REF (default: HEAD), --json |
| hooks install | Install agent instructions and/or git hooks | --claude, --cursor, --copilot, --precommit, --claude-code-hook |
| hooks uninstall | Remove previously installed hooks | --claude, --cursor, --copilot, --precommit, --claude-code-hook |
| hooks status | Show installation status of all hooks | — |
| compact | Compact changelog tables by summarizing old entries | --keep N (default 10), --dry-run |
| archive-tasks | Move completed task items to archive section | --dry-run |
| view | Filter spec content by stakeholder role | --role (dev\|qa\|product\|agent), --spec PATH |
| merge | Auto-resolve git merge conflicts in spec files | --dry-run, --all, --json |
| issues | Verify GitHub issue references in spec frontmatter | --create (create drift issues for failures) |

### Global Flags

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| --strict | bool | false | Treat warnings as errors (exit 1) |
| --require-coverage | Option usize | None | Fail if file coverage percent is below threshold |
| --root | Option PathBuf | cwd | Project root directory |
| --format | text\|json\|markdown | text | Output format: colored text, machine-readable JSON, or markdown |
| --json | bool | false | Shorthand for `--format json` |
| --force | bool | false | Bypass hash cache and re-validate all specs |

### Internal Functions

All functions in main.rs are private (no pub keyword). Key internal functions:

- **main** — Parse CLI args, canonicalize root, dispatch to subcommand handler
- **cmd_init** — Create specsync.json with auto-detected source dirs; no-op if config exists
- **cmd_check** — Load config, discover specs, validate, print results, exit with status
- **cmd_coverage** — Load config, compute coverage, print detailed coverage report
- **cmd_generate** — Scaffold specs for unspecced modules; optionally use AI provider
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
- **auto_fix_specs** — Scan source files for undocumented exports and auto-add stubs to spec Public API tables
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
2. `--root` defaults to the current working directory; the path is canonicalized
3. `--strict` causes warnings to produce a non-zero exit code
4. `--require-coverage N` causes exit 1 if file coverage percent < N
5. `--json` switches all output to machine-readable JSON (no ANSI colors)
6. `cmd_init` is idempotent — does nothing if `specsync.json` or `.specsync.toml` already exists
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

- **Given** `specsync.json` already exists in the project root
- **When** `specsync init` is run
- **Then** prints "specsync.json already exists" and returns without modifying it

### Scenario: Coverage threshold

- **Given** file coverage is 80%
- **When** `specsync check --require-coverage 90` is run
- **Then** the process exits with code 1 and prints the unspecced files

### Scenario: Generate with AI

- **Given** an AI provider is available
- **When** `specsync generate --provider auto` is run
- **Then** auto-detects the provider and generates AI-enhanced specs

### Scenario: Resolve without network

- **Given** specs have cross-project `depends_on` refs
- **When** `specsync resolve` is run (without `--remote`)
- **Then** lists the refs but does not verify them against remote registries

### Scenario: Fix auto-adds undocumented exports

- **Given** a spec's source files have exports not documented in the Public API section
- **When** `specsync check --fix` is run
- **Then** stub rows (`| \`name\` | <!-- TODO: describe --> |`) are appended to the Public API section and the spec file is written to disk

### Scenario: Fix does not duplicate already-documented exports

- **Given** a spec already documents `login` but not `logout`
- **When** `specsync check --fix` is run
- **Then** only `logout` is added; `login` is not duplicated

### Scenario: Fix creates Public API section when missing

- **Given** a spec has no `## Public API` section
- **When** `specsync check --fix` is run
- **Then** a new `## Public API` section with a table header and stub rows is appended to the spec

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
| AI provider not found (with `--provider`) | Prints error to stderr and exits 1 |
| Failed to write `specsync.json` | Panics with "Failed to write specsync.json" |
| Failed to create spec directory | Prints error to stderr and exits 1 |
| Failed to write spec file | Prints error to stderr and exits 1 |
| Failed to write `specsync-registry.toml` | Prints error to stderr and exits 1 |
| No spec files found (non-generate commands) | Prints guidance message and exits 0 |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| config | `load_config`, `detect_source_dirs` |
| parser | `parse_frontmatter` |
| validator | `validate_spec`, `find_spec_files`, `compute_coverage`, `get_schema_table_names`, `is_cross_project_ref`, `parse_cross_project_ref` |
| exports | `has_extension`, `get_exported_symbols` (used by auto_fix_specs and cmd_diff) |
| generator | `generate_specs_for_unspecced_modules`, `generate_specs_for_unspecced_modules_paths`, `generate_companion_files_for_spec` |
| ai | `resolve_ai_provider` |
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

### Consumed By

| Module | What is used |
|--------|-------------|
| (none) | `main.rs` is the top-level entry point — nothing imports it |

## Change Log

| Date | Change |
|------|--------|
| 2026-03-25 | Initial spec |
| 2026-04-06 | Add compact, archive-tasks, view, merge, issues subcommands; add --force, --create-issues, --format flags; add hash_cache/github/archive/compact/view/merge dependencies |
