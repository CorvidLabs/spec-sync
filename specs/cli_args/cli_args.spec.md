---
module: cli_args
version: 2
status: stable
files:
  - src/cli.rs
db_tables: []
tracks: []
depends_on:
  - specs/types/types.spec.md
---

# CLI Args

## Purpose

Defines the CLI argument parser using Clap derive macros. Declares all subcommands, their flags, and global options for the `specsync` binary. This is the single source of truth for the CLI surface area — every flag, argument, and subcommand enum variant lives here.

## Public API

### Exported Structs

| Type | Description |
|------|-------------|
| `Cli` | Top-level Clap parser struct with global flags (`--strict`, `--root`, `--format`, `--json`, `--enforcement`, `--require-coverage`) and a subcommand field |

### Exported Enums

| Type | Description |
|------|-------------|
| `Command` | Root subcommand enum including canonical validation, module lifecycle, and verified SDD Change operations |
| `HooksAction` | Sub-subcommand for `Hooks`: Install, Uninstall, Status — each with boolean flags for target selection (claude, cursor, copilot, agents, precommit, claude_code_hook) |
| `AgentsAction` | Sub-subcommand for `Agents`: Install, Uninstall, Status — each with boolean flags for target selection (claude, cursor, codex, gemini) |
| `LifecycleAction` | Sub-subcommand for `Lifecycle`: Promote, Demote, Set, Status, History, Guard, AutoPromote, Enforce — manages spec lifecycle transitions |
| `ChangeAction` | Sub-subcommand for `Change`: New, Answer, Depend, List, Show, Status, Approve, Start, Verify, Accept, Archive, Check, Adopt |

## Invariants

1. All global flags use `#[arg(global = true)]` so they work regardless of subcommand position
2. `--json` is a shorthand alias for `--format json` — both set the same output format
3. `--enforcement` accepts three modes matching `types::EnforcementMode`: warn, enforce-new, strict
4. Default output format is `text` when neither `--json` nor `--format` is specified
5. The `Command` enum is optional — running `specsync` with no subcommand defaults to `Check`
6. Each `HooksAction::Install` / `Uninstall` variant carries identical boolean flags for symmetric install/uninstall
7. Each `AgentsAction::Install` / `Uninstall` variant carries identical boolean flags for symmetric install/uninstall, mirroring `HooksAction`

## Behavioral Examples

### Scenario: Global strict flag propagates to subcommand

- **Given** user runs `specsync check --strict`
- **When** Clap parses arguments
- **Then** `Cli.strict == true` is accessible regardless of the `Check` subcommand

### Scenario: Default subcommand

- **Given** user runs `specsync` with no subcommand
- **When** Clap parses arguments
- **Then** `Cli.command` is `None`, and `main.rs` defaults to Check behavior

### Scenario: Hooks install targets

- **Given** user runs `specsync hooks install --claude --precommit`
- **When** Clap parses arguments
- **Then** `HooksAction::Install { claude: true, precommit: true, ... }` with all others false

### Scenario: Agents install targets

- **Given** user runs `specsync agents install --claude --gemini`
- **When** Clap parses arguments
- **Then** `AgentsAction::Install { claude: true, gemini: true, cursor: false, codex: false }`

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Unknown subcommand | Clap prints error with usage help and exits non-zero |
| Missing required argument (e.g., `new` without name) | Clap prints error listing required args |
| Invalid `--enforcement` value | Clap prints accepted values: warn, enforce-new, strict |
| Invalid `--format` value | Clap prints accepted values: text, json, markdown |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| types | `OutputFormat`, `EnforcementMode` enum types for flag parsing |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | `Cli::parse()` to drive the entire application |
| cmd_hooks | `HooksAction` enum for hooks subcommand dispatch |
| cmd_agents | `AgentsAction` enum for agents subcommand dispatch |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
| 2026-04-11 | Add LifecycleAction enum and Lifecycle command variant |
| 2026-07-01 | Add AgentsAction enum and Agents command variant |
| 2026-07-10 | Add ChangeAction and the 5.0 SDD lifecycle namespace |
