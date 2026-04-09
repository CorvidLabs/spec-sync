---
module: cmd_hooks
version: 1
status: stable
files:
  - src/commands/hooks.rs
db_tables: []
tracks: []
depends_on:
  - specs/hooks/hooks.spec.md
  - specs/cli_args/cli_args.spec.md
---

# Cmd Hooks

## Purpose

Implements the `specsync hooks` command. Routes install/uninstall/status subcommands to the hooks library module, translating boolean CLI flags into hook target lists.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_hooks` | `root: &Path, action: HooksAction` | `()` | Dispatch hooks action to library module |

## Invariants

1. Maps `HooksAction` variants to `hooks::cmd_install`, `cmd_uninstall`, `cmd_status`
2. Private `collect_hook_targets()` converts boolean flags to target vec
3. When no target flags set, installs/uninstalls all targets

## Behavioral Examples

### Scenario: Install specific hooks

- **Given** `specsync hooks install --claude --precommit`
- **When** `cmd_hooks` runs
- **Then** installs only CLAUDE.md and pre-commit hook

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Hook write fails | Delegated to hooks module |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| hooks | `cmd_install`, `cmd_uninstall`, `cmd_status` |
| cli_args | `HooksAction` enum |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync hooks` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
