---
module: commands
version: 10
status: stable
files:
  - src/commands/mod.rs
db_tables: []
tracks: []
depends_on:
  - specs/agents/agents.spec.md
  - specs/change/change.spec.md
  - specs/changelog/changelog.spec.md
  - specs/comment/comment.spec.md
  - specs/compact/compact.spec.md
  - specs/config/config.spec.md
  - specs/deps/deps.spec.md
  - specs/github/github.spec.md
  - specs/hooks/hooks.spec.md
  - specs/ignore/ignore.spec.md
  - specs/merge/merge.spec.md
  - specs/parser/parser.spec.md
  - specs/rehash/rehash.spec.md
  - specs/schema/schema.spec.md
  - specs/scoring/scoring.spec.md
  - specs/types/types.spec.md
  - specs/validator/validator.spec.md
  - specs/view/view.spec.md
---

# Commands

## Purpose

Shared command infrastructure and registry used by all CLI subcommands. It centralizes config loading, spec discovery, filtering, schema construction, validation, exit handling, GitHub drift issues, and dispatch modules including the verified 5.0 change lifecycle.

## Public API

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `load_and_discover` | `root: &Path, allow_empty: bool` | `(SpecSyncConfig, Vec<PathBuf>)` | Load config and discover all spec files (excluding `_`-prefixed); exits if empty and `allow_empty` is false |
| `validate_module_name` | `module_name: &str` | `Result<(), String>` | Validate a user-supplied module name as one portable path segment: reject traversal/control/Windows-invalid characters, trailing spaces/dots, Windows reserved device basenames (including before extensions), and names whose `<name>.spec.md` filename exceeds 255 UTF-8 bytes |
| `filter_specs` | `root: &Path, spec_files: &[PathBuf], filters: &[String]` | `Vec<PathBuf>` | Filter spec files by user-provided names/paths (exact path, relative path, filename, module name); returns all if filters is empty |
| `filter_by_status` | `spec_files: &[PathBuf], exclude: &[String], only: &[String]` | `Vec<PathBuf>` | Filter spec files by their frontmatter status field; supports exclude-list and allow-list modes |
| `build_schema_columns` | `root: &Path, config: &SpecSyncConfig` | `HashMap<String, SchemaTable>` | Build column-level schema from migration files if `schema_dir` is configured |
| `run_validation` | `root: &Path, spec_files: &[PathBuf], ownership_spec_files: &[PathBuf], schema_tables: &HashSet<String>, schema_columns: &HashMap<String, schema::SchemaTable>, config: &types::SpecSyncConfig, collect: bool, explain: bool, ignore_rules: &IgnoreRules` | `(usize, usize, usize, usize, Vec<String>, Vec<String>, Vec<String>)` | Run validation on all spec files returning (errors, warnings, passed, total, error_strings, warning_strings, notice_strings); contains full text rendering logic |
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

## Invariants

1. `load_and_discover` excludes spec files starting with `_` (underscore prefix marks internal/template specs)
2. `filter_specs` matches against four forms: exact path, relative path, filename stem, and module name (stem minus `.spec` suffix)
3. `run_validation` applies ignore rules (global, inline, per-spec) to filter warnings before counting
4. In text mode, draft specs show explicit "Section validation skipped (status: draft)" and "Export validation skipped (status: draft)" notices instead of misleading green checkmarks, plus a closing hint to set `status: active`
5. Failing checks render negated labels (e.g. "✗ Frontmatter invalid"), never a ✗ next to a passing label
4. Exit code logic by enforcement mode: Warn → always 0; EnforceNew → 1 if unspecced files; Strict → 1 on errors, also 1 on warnings when `--strict`
5. `--require-coverage N` triggers exit 1 if file coverage percent < N regardless of enforcement mode. When `N > 0` but 0 source files were discovered (empty/misconfigured `source_dirs` or an over-broad `exclude_patterns`), the gate fails loud (exit 1) rather than passing on the vacuous 100% reported for an empty source tree
6. `create_drift_issues` groups errors by spec path and creates one GitHub issue per spec, not per error
7. Drift-creation terminal output never emits untrusted repository-resolution errors, spec paths,
   issue URLs, or provider errors without diagnostic sanitization; delegated GitHub issue
   title/body construction applies its own hostile-text sanitization
8. `validate_module_name` is platform-independent: every host rejects Windows-invalid characters,
   reserved device basenames, trailing spaces/dots, and module names longer than 247 UTF-8 bytes so
   generated spec filenames remain portable 255-byte path components.

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

### Scenario: Hostile drift-creation text

- **Given** repository, spec-path, provider-error, or created-issue URL text contains terminal
  controls, bidirectional formatting, or Unicode line/paragraph separators
- **When** `create_drift_issues()` reports resolution, success, or failure
- **Then** terminal output contains only sanitized diagnostics, and the delegated GitHub issue
  title/body cannot preserve hostile formatting characters

## Error Cases

| Condition | Behavior |
|-----------|----------|
| No spec files found and `allow_empty` is false | Prints suggestion to run `specsync generate` and exits 0 |
| Filter matches no specs | Prints warning listing unmatched filters, returns empty vec (cmd_check then exits 1) |
| `schema_dir` not configured | `build_schema_columns` returns empty map (no error) |
| GitHub repo unresolvable for drift issues | Prints a sanitized error and returns without creating issues |
| `gh` CLI fails to create issue | Prints a sanitized per-spec error and continues with remaining specs |
| Module name is non-portable, reserved, or too long | Returns an actionable `Err` before any output path is joined or created |

## Dependencies

**Consumes**

| Module | What is used |
|--------|-------------|
| config | `load_config` |
| ignore | `IgnoreRules`, `parse_inline` |
| schema | `SchemaTable`, `build_schema` |
| scoring | `score_spec` (when explain mode) |
| types | `SpecSyncConfig`, `CoverageReport`, `EnforcementMode`, `OutputFormat` |
| validator | `find_spec_files`, `validate_spec` |
| github | `resolve_repo`, `create_drift_issue` |

**Consumed By**

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
| cmd_stale | `load_and_discover` |
| cmd_diff | `load_and_discover` |

**Frontmatter Synchronization**

Implementation SHALL add these canonical dependency specs to `depends_on`: `specs/agents/agents.spec.md`, `specs/change/change.spec.md`, `specs/changelog/changelog.spec.md`, `specs/comment/comment.spec.md`, `specs/compact/compact.spec.md`, `specs/deps/deps.spec.md`, `specs/hooks/hooks.spec.md`, `specs/merge/merge.spec.md`, `specs/parser/parser.spec.md`, `specs/rehash/rehash.spec.md`, `specs/view/view.spec.md`. This YAML frontmatter update is an explicit implementation edit because semantic section deltas do not apply frontmatter.

## Change Log

| Date | Change |
|------|--------|
| 2026-07-22 | v9 / CHG-0063: sanitize hostile terminal diagnostics throughout drift creation, preserve the public rendered `Vec<String>` API, and use private structured exact-path attribution for GitHub drift creation |
| 2026-07-23 | v10 / CHG-0063: enforce portable module components, Windows reserved-name rules, and the 255-byte generated spec filename limit on every platform |
| 2026-07-01 | v4: Add `agents` submodule (native AI-tool skill/slash-command installation) |
| 2026-07-10 | v5: Add `change` submodule for the verified SDD lifecycle |
| 2026-06-11 | v3: Partial export-coverage summary ("N/M exports documented") prints as ⚠ — it is counted as a warning, so the summary's warning count now matches the printed ⚠ lines |
| 2026-06-11 | v2: Draft specs report skipped section/export validation explicitly; failing frontmatter renders a negated label |
| 2026-04-09 | Initial spec |
| 2026-04-11 | Add lifecycle submodule and filter_by_status function |
| 2026-07-11 | CHG-0003-finalize-specsync-5-0-release-consistency-and-parallel-validation: Finalize SpecSync 5.0 release consistency and parallel validation |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-14 | CHG-0039-allow-draft-specs-to-declare-planned-missing-source-mappings-without-failing-str: Allow draft specs to declare planned missing source mappings without failing strict validation while preserving path safety ownership enforcement exact coverage and complete notice contracts |
