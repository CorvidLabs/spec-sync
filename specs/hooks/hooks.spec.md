---
module: hooks
version: 4
status: stable
files:
  - src/hooks.rs
db_tables: []
tracks: [39]
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
3. Shared pre-commit files use balanced project-keyed managed markers; partial or reversed marker pairs fail before mutation
4. Pre-commit hook is made executable (mode 0o755) on Unix
5. Uninstalling Claude Code hook settings is refused — must be done manually (too risky to auto-edit)
6. Empty targets list means "all targets"
7. Pre-commit hook resolution honors Git worktrees, submodules, and `core.hooksPath`; installation inserts its strict blocking block before a trailing `exit 0`
8. `cmd_install` exits with code 1 if any hook installation fails
9. Uninstall removes only the exact current-project block and preserves user and other-project content

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

### Scenario: Shared hook directory

- **Given** two projects resolve to the same `core.hooksPath`
- **When** each installs its pre-commit integration
- **Then** each project receives a distinct managed block and uninstalling either leaves the other intact

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Cannot read/write file | Returns `Err` with descriptive message |
| Cannot create directory | Returns `Err` with descriptive message |
| Uninstall Claude Code hook | Returns `Err` — must be removed manually |
| Cannot parse existing settings.json | Returns `Err` with parse error |
| Partial or reversed pre-commit managed markers | Returns `Err` without rewriting the hook |
| Project is not inside a Git repository | Pre-commit installation returns an actionable `Err` |

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
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-30 | CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara: Stabilize SpecSync 6.0 with one scope approval, same-PR finalization, lightweight archive CI, scoped review, and selected UX fixes |
| 2026-07-31 | CHG-0069-scoped-change-check-change-audit-and-agent-pack-for-the-two-verb-lifecycle: Scoped change check, change audit, and agent pack for the two-verb lifecycle |
