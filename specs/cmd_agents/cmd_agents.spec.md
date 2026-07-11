---
module: cmd_agents
version: 2
status: stable
files:
  - src/commands/agents.rs
db_tables: []
depends_on:
  - specs/agents/agents.spec.md
  - specs/cli_args/cli_args.spec.md
---

# Cmd Agents

## Purpose

Implements `specsync agents` by routing install, uninstall, and status actions for the project-local verified-SDD skills and supported create-spec/create-change commands.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_agents` | `root: &Path, action: AgentsAction` | `()` | Dispatch agents action to library module |

### Exported Types

| Type | Description |
|------|-------------|

## Invariants

1. Maps `AgentsAction` variants to `agents::cmd_install`, `cmd_uninstall`, `cmd_status`
2. Private `collect_agent_targets()` converts boolean flags (`--claude`/`--cursor`/`--codex`/`--gemini`) to a target vec
3. When no target flags are set, installs/uninstalls all tools

## Behavioral Examples

### Scenario: Install specific tools

- **Given** `specsync agents install --claude --gemini`
- **When** `cmd_agents` runs
- **Then** installs only Claude Code's and Gemini CLI's integrations

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Skill/command write fails | Delegated to `agents` module |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| agents | `cmd_install`, `cmd_uninstall`, `cmd_status`, `AgentTool` |
| cli_args | `AgentsAction` enum |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync agents` |

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-07-01 | claude | Initial spec |
| 2026-07-11 | CHG-0003-finalize-specsync-5-0-release-consistency-and-parallel-validation: Finalize SpecSync 5.0 release consistency and parallel validation |
