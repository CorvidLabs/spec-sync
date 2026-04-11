---
module: cmd_lifecycle
version: 1
status: stable
files:
  - src/commands/lifecycle.rs
db_tables: []
tracks: []
depends_on:
  - specs/commands/commands.spec.md
  - specs/parser/parser.spec.md
  - specs/scoring/scoring.spec.md
  - specs/types/types.spec.md
---

# Cmd Lifecycle

## Purpose

Implements the `specsync lifecycle` command. Manages spec status transitions — promote, demote, set, status, history, and guard evaluation. Validates transitions against the `SpecStatus` lifecycle graph with configurable transition guards and writes updated frontmatter to disk.

## Public API

### Exported Structs

| Type | Description |
|------|-------------|
| `GuardResult` | Result of evaluating transition guards — `passed: bool` and `failures: Vec<String>` |

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_promote` | `root: &Path, spec_filter: &str, format: OutputFormat, force: bool` | `()` | Advance a spec to its next lifecycle status (draft→review→active→stable) |
| `cmd_demote` | `root: &Path, spec_filter: &str, format: OutputFormat, force: bool` | `()` | Move a spec back to its previous lifecycle status |
| `cmd_set` | `root: &Path, spec_filter: &str, target_status: &str, format: OutputFormat, force: bool` | `()` | Set a spec to any valid status with transition validation |
| `cmd_status` | `root: &Path, spec_filter: Option<&str>, format: OutputFormat` | `()` | Display lifecycle status of one or all specs |
| `cmd_history` | `root: &Path, spec_filter: &str, format: OutputFormat` | `()` | Display lifecycle transition history for a spec |
| `cmd_guard` | `root: &Path, spec_filter: &str, target_str: Option<&str>, format: OutputFormat` | `()` | Evaluate and display guard results for a spec transition |
| `cmd_auto_promote` | `root: &Path, format: OutputFormat, dry_run: bool` | `()` | Scan all specs and promote any that pass guards automatically |
| `cmd_enforce` | `root: &Path, format: OutputFormat, require_status: bool, check_max_age: bool` | `()` | Enforce lifecycle policies — flag specs without status or stuck too long in a phase |
| `evaluate_guards` | `root: &Path, spec_path: &Path, config: &SpecSyncConfig, from: &SpecStatus, to: &SpecStatus` | `GuardResult` | Evaluate all transition guards for a status change |

## Invariants

1. Promote/demote use `SpecStatus::next()` / `SpecStatus::prev()` for linear transitions
2. `set` validates transitions via `SpecStatus::can_transition_to()` unless `--force` is used
3. Status updates are written in-place by regex-replacing the `status:` line within frontmatter delimiters only (never in body)
4. Single spec is resolved via `filter_specs` — exits 1 if ambiguous or no match
5. JSON output uses `OutputFormat::Json` for machine-readable results
6. Transition guards check min_score, require_sections, and staleness
7. Lifecycle history is appended to frontmatter `lifecycle_log` when `track_history` is enabled

## Behavioral Examples

### Scenario: Promote draft to review

- **Given** spec `auth` has `status: draft`
- **When** `cmd_promote(root, "auth", Text, false)` runs
- **Then** updates `auth.spec.md` to `status: review`

### Scenario: Guard blocks transition

- **Given** transition guard requires min_score of 60
- **When** spec has score 45
- **Then** prints guard failure and exits 1

### Scenario: Status of all specs

- **Given** multiple specs with various statuses
- **When** `cmd_status(root, None, Text)` runs
- **Then** prints specs grouped by status with colored labels

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Spec filter matches no specs | Exits 1 with error message |
| Ambiguous spec filter (multiple matches) | Exits 1, lists all matches |
| No `status:` line in frontmatter | Prints error, exits 1 |
| Invalid transition (without `--force`) | Prints error with valid alternatives, exits 1 |
| Guard check fails (without `--force`) | Prints guard failures, exits 1 |
| File write fails | Prints error, exits 1 |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| commands | `load_and_discover`, `filter_specs` |
| parser | `parse_frontmatter`, `get_missing_sections` |
| scoring | `score_spec` |
| types | `SpecStatus`, `OutputFormat`, `SpecSyncConfig`, `LifecycleConfig`, `TransitionGuard` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync lifecycle` subcommands |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-11 | Initial spec |
| 2026-04-11 | Add cmd_auto_promote, cmd_enforce to API table; fix invariant #3 scope |
