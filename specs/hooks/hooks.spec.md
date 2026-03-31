---
module: hooks
version: 1
status: stable
files:
  - src/hooks.rs
db_tables: []
depends_on: []
---

# Hooks

## Purpose

Manages agent instruction files and git hooks for spec-sync integration. Installs and uninstalls instruction snippets for Claude (CLAUDE.md), Cursor (.cursorrules), Copilot (.github/copilot-instructions.md), Agents (AGENTS.md), a git pre-commit hook, and Claude Code settings.json hooks.

## Public API

### Exported Enums

| Type | Description |
|------|-------------|
| `HookTarget` | All installable hook targets: Claude, Cursor, Copilot, Agents, Precommit, ClaudeCodeHook |

### Exported HookTarget Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `all` | — | `&'static [HookTarget]` | Returns slice of all hook targets |
| `name` | `&self` | `&'static str` | Short name string for this target (e.g. "claude", "precommit") |
| `description` | `&self` | `&'static str` | Human-readable description (e.g. "CLAUDE.md agent instructions") |
| `from_str` | `s: &str` | `Option<Self>` | Parse a hook target from string (case-insensitive, aliases supported) |

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `is_installed` | `root, target` | `bool` | Check if a specific hook target is already installed |
| `install_hook` | `root, target` | `Result<bool, String>` | Install a single hook target; returns `Ok(false)` if already present |
| `uninstall_hook` | `root, target` | `Result<bool, String>` | Uninstall a single hook target; returns `Ok(false)` if not found |
| `cmd_install` | `root, targets` | `()` | CLI handler: install specified targets (or all if empty) |
| `cmd_uninstall` | `root, targets` | `()` | CLI handler: uninstall specified targets (or all if empty) |
| `cmd_status` | `root` | `()` | CLI handler: show installation status of all hook targets |

## Invariants

1. Installation is idempotent — re-installing an already-installed hook is a no-op returning `Ok(false)`
2. Agent instruction files are appended to existing files, not overwritten
3. Installation checks for existing spec-sync content by marker strings ("Spec-Sync Integration", "Spec-Sync Rules")
4. Pre-commit hook is made executable (mode 0o755) on Unix
5. Uninstalling Claude Code hook settings is refused — must be done manually (too risky to auto-edit)
6. Empty targets list means "all targets"
7. Pre-commit hook appends to existing hooks (skipping shebang) rather than replacing them
8. `cmd_install` exits with code 1 if any hook installation fails

## Behavioral Examples

### Scenario: Install all hooks

- **Given** a project with no hooks installed
- **When** `cmd_install(root, &[])` is called
- **Then** installs CLAUDE.md, .cursorrules, copilot-instructions.md, AGENTS.md, pre-commit hook, and Claude Code settings

### Scenario: Already installed

- **Given** CLAUDE.md already contains "Spec-Sync Integration"
- **When** `install_hook(root, HookTarget::Claude)` is called
- **Then** returns `Ok(false)` without modifying the file

### Scenario: Uninstall cursor rules

- **Given** .cursorrules contains the spec-sync section
- **When** `uninstall_hook(root, HookTarget::Cursor)` is called
- **Then** removes the spec-sync section, returns `Ok(true)`; deletes the file if it becomes empty

### Scenario: Check status

- **Given** Claude and Precommit hooks are installed, others are not
- **When** `cmd_status(root)` is called
- **Then** shows "installed" for Claude and Precommit, "not installed" for the rest

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Cannot read/write file | Returns `Err` with descriptive message |
| Cannot create directory | Returns `Err` with descriptive message |
| Uninstall Claude Code hook | Returns `Err` — must be removed manually |
| Cannot parse existing settings.json | Returns `Err` with parse error |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| serde_json | JSON parsing for Claude Code settings.json |
| colored | Terminal output formatting |

### Consumed By

| Module | What is used |
|--------|-------------|
| main | `cmd_install`, `cmd_uninstall`, `cmd_status`, `HookTarget` |

## Change Log

| Date | Change |
|------|--------|
| 2026-03-25 | Initial spec |
| 2026-03-30 | Add Agents (AGENTS.md) hook target |
