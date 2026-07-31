---
module: cli_args
version: 16
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

Defines the complete CLI argument grammar, including the discoverable one-approval change path and compatible historical repair commands.

## Public API

**Exported Structs**

| Type | Description |
|------|-------------|
| `Cli` | Top-level Clap parser struct with global flags (`--strict`, `--root`, `--format`, `--json`, `--enforcement`, `--require-coverage`) and a subcommand field |

**Exported Enums**

| Type | Description |
|------|-------------|
| `Command` | Root subcommand enum including canonical validation, module lifecycle, and verified SDD Change operations |
| `HooksAction` | Sub-subcommand for `Hooks`: Install, Uninstall, Status — each with boolean flags for target selection (claude, cursor, copilot, agents, precommit, claude_code_hook) |
| `AgentsAction` | Sub-subcommand for `Agents`: Install, Uninstall, Status — each with boolean flags for target selection (claude, cursor, codex, gemini) |
| `LifecycleAction` | Sub-subcommand for `Lifecycle`: Promote, Demote, Set, Status, History, Guard, AutoPromote, Enforce — manages spec lifecycle transitions |
| `ChangeAction` | Sub-subcommand for `Change`: New, Answer, Depend, Supersede, List, Show, Status, Approve, Check, Review with explicit pass/block verdict, Finalize, plus compatible historical repair/transition commands |

## Invariants

1. All global flags remain available around subcommands.
2. `--json` remains an alias for JSON output.
3. MCP is read-only unless the `Mcp` command's `--allow-write` flag is present.
4. Existing deterministic generation and verified SDD command grammar remains available.
5. `change finalize` prepares the current PR and has no merge/provider arguments.
6. Global `--strict` adds validators to the same change workflow rather than selecting another state machine.
7. `change review` accepts a stable ASCII reviewer claim, defaults to a passing verdict, and accepts explicit `--verdict pass|block`.

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

### Scenario: Explicit MCP write authorization

- **Given** user runs `specsync mcp --allow-write`
- **When** Clap parses arguments
- **Then** `Command::Mcp { allow_write: true }` is dispatched; omitting the flag yields `false`

### Scenario: Single change workflow

- **Given** a newcomer follows command help
- **When** they inspect `specsync change --help`
- **Then** `new`, `approve`, `check`, `status`, and `finalize` are plain discoverable commands and no SpecSync merge command is offered

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Unknown subcommand | Clap prints error with usage help and exits non-zero |
| Missing required argument (e.g. `new` without name) | Clap prints error listing required args |
| Invalid `--enforcement` value | Clap prints accepted values: warn, enforce-new, strict |
| Invalid `--format` value | Clap prints accepted values: text, json, markdown, github, table, csv |
| Unsupported MCP option or value | Clap rejects it before starting the stdio server |
| `change reopen` without `--actor` or `--reason` | Clap names the missing required argument and exits non-zero |
| `change correct-owner` without actor, reason, or any batch selection | Clap names the missing required argument and exits non-zero |
| `change correct-owner` with conflicting `--all-missing`, `--manifest`, and `--path` modes | Clap rejects the conflicting selection before domain mutation |
| `change review` with an unknown verdict | Clap lists `pass` and `block` and exits before evidence mutation |

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
| 2026-07-11 | CHG-0003-finalize-specsync-5-0-release-consistency-and-parallel-validation: Finalize SpecSync 5.0 release consistency and parallel validation |
| 2026-07-11 | CHG-0007-harden-specsync-5-0-as-an-agent-native-secret-free-sdd-core-and-close-release-r: Harden SpecSync 5.0 as an agent-native, secret-free SDD core and close release regressions |
| 2026-07-11 | CHG-0012-correct-specsync-5-0-documentation-cli-help-and-hub-deep-links: Correct initialization and full-scaffold help while preserving the CLI grammar |
| 2026-07-11 | CHG-0012-correct-specsync-5-0-documentation-cli-help-and-hub-deep-links: Correct SpecSync 5.0 documentation, CLI help, and hub deep links |
| 2026-07-13 | Add required actor/reason grammar for audited stale-accepted reopen |
| 2026-07-13 | CHG-0015-add-audited-stale-accepted-change-reopening: Add audited stale accepted change reopening |
| 2026-07-15 | CHG-0040-support-audited-append-only-correction-of-accepted-interview-metadata-without-re: Support audited append-only correction of accepted interview metadata without replaying canonical deltas |
| 2026-07-15 | CHG-0043-make-accepted-change-validity-successor-aware-with-exact-per-input-evidence-rec: Make accepted-change validity successor-aware with exact per-input evidence, recursive cycle-safe validation, fail-closed legacy compatibility, and safe archived successors |
| 2026-07-15 | CHG-0047-permit-audited-deterministic-ownership-corrections-for-reopened-already-applied: Permit audited deterministic ownership corrections for reopened already-applied changes |
| 2026-07-19 | CHG-0055-batch-mode-for-change-correct-owner-so-multiple-omitted-exact-canonical-owners-c: Batch mode for change correct-owner so multiple omitted exact canonical owners can be audited and appended in one transactional correction before a single reapprove-verify-accept cycle |
| 2026-07-19 | CHG-0057-add-a-native-migration-path-for-5-0-1-era-change-ledgers-that-backfills-the-5-1: Add a native migration path for 5.0.1-era change ledgers that backfills the 5.1 reopening stale and current acceptance-input digest fields idempotently with a closing-digest verification pass, and surfaces an actionable migrate hint when check encounters the 5.0.1 reopening schema |
| 2026-07-21 | CHG-0062: Add explicit `mcp --allow-write` authorization while preserving a read-only default |
| 2026-07-22 | CHG-0062-harden-mcp-root-confinement-write-authorization-argument-validation-and-notif: Harden MCP root confinement, write authorization, argument validation, and notification semantics for issue 414 |
| 2026-07-30 | CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara: Stabilize SpecSync 6.0 with one scope approval, same-PR finalization, lightweight archive CI, scoped review, and selected UX fixes |
| 2026-07-30 | CHG-0068: Add a discoverable optional `change review --verdict pass|block` conclusion |
| 2026-07-30 | CHG-0068: Clarify reviewer text as a stable claim while hosted check provenance authenticates the append-only review trail |
