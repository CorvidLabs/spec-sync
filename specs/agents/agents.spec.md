---
module: agents
version: 2
status: stable
files:
  - src/agents.rs
db_tables: []
depends_on: []
---

# Agents

## Purpose

Installs native, tool-owned skill and slash-command files for AI coding agents (Claude Code, Cursor, Codex, Gemini CLI), distinct from the prose-instruction-file mechanism in `hooks.rs`. Ships a `SKILL.md` each tool auto-discovers and a `/specsync:create-spec` (or tool-equivalent) slash command that scaffolds a spec from a module name or a natural-language feature description.

## Public API

### Exported Enums

| Type | Description |
|------|-------------|
| `AgentTool` | AI coding tools that receive native skill/command installation: Claude, Cursor, Codex, Gemini |

### Exported AgentTool Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `all` | — | `&'static [AgentTool]` | Returns slice of all four agent tools |
| `name` | `&self` | `&'static str` | Short name string for this tool (e.g. "claude", "gemini") |
| `description` | `&self` | `&'static str` | Human-readable description of what gets installed for this tool |
| `from_str` | `s: &str` | `Option<Self>` | Parse an agent tool from string (case-insensitive) |

### Exported Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `is_installed` | `(root: &Path, tool: AgentTool) -> bool` | True if every artifact this tool should have (skill dir and/or command file) already exists |
| `install_agent` | `(root: &Path, tool: AgentTool) -> Result<bool, String>` | Install skill + command files for one tool; `Ok(false)` if everything was already present |
| `uninstall_agent` | `(root: &Path, tool: AgentTool) -> Result<bool, String>` | Remove a tool's skill dir and/or command file; `Ok(false)` if nothing was present |
| `cmd_install` | `(root: &Path, targets: &[AgentTool])` | CLI handler: install specified tools (or all if empty) |
| `cmd_uninstall` | `(root: &Path, targets: &[AgentTool])` | CLI handler: uninstall specified tools (or all if empty) |
| `cmd_status` | `(root: &Path)` | CLI handler: show installation status of all agent tools |

## Invariants

1. Each `AgentTool` variant owns zero, one, or two artifacts: a skill directory (`<tool-dir>/skills/spec-sync/SKILL.md`) and/or a `create-spec` command file — Codex has skill only (its command mechanism is deprecated and global-only, outside any project root), all others have both.
2. Installation is idempotent per-artifact and content-aware — `install_agent` writes an artifact when it's missing *or* when its existing content differs from the current template (so upgrading spec-sync refreshes stale installations), and returns `Ok(false)` only when every artifact already matches the current template exactly.
3. Every artifact spec-sync writes lives inside a `spec-sync/`-named skill folder or a `specsync`-namespaced command file/directory that spec-sync fully owns — no marker-string surgery on shared files is needed (unlike `hooks.rs`).
4. `uninstall_agent` removes the skill directory wholesale (`remove_dir_all`) and the command file, then removes the command file's immediate parent directory only if that parent is named `specsync` and is now empty — it never removes a tool's shared `commands/` directory (e.g. `.claude/commands/`, `.cursor/commands/`), which may hold unrelated user commands.
5. Cursor's command file is flat (`.cursor/commands/specsync-create-spec.md`, no namespaced subdirectory, no YAML frontmatter) since Cursor's command mechanism doesn't support either.
6. Claude and Gemini's commands live in a namespaced subdirectory (`.claude/commands/specsync/create-spec.md`, `.gemini/commands/specsync/create-spec.toml`) so they're invoked as `/specsync:create-spec`.
7. Gemini's command file is TOML (`description`/`prompt` keys, `{{args}}` placeholder), hand-built as a string template since no `toml` crate dependency exists in this project.
8. Empty targets list means "all tools", matching the `hooks` module's convention.
9. `cmd_install` exits with code 1 if any tool installation fails.

## Behavioral Examples

### Scenario: Install all agent tools

- **Given** a project with no agent integrations installed
- **When** `cmd_install(root, &[])` is called
- **Then** installs Claude's skill + command, Cursor's skill + command, Codex's skill, and Gemini's skill + command

### Scenario: Already installed

- **Given** `.claude/skills/spec-sync/SKILL.md` and `.claude/commands/specsync/create-spec.md` both exist
- **When** `install_agent(root, AgentTool::Claude)` is called
- **Then** returns `Ok(false)` without modifying either file

### Scenario: Uninstall preserves sibling commands

- **Given** `.claude/commands/specsync/create-spec.md` and an unrelated `.claude/commands/other-command.md` both exist
- **When** `uninstall_agent(root, AgentTool::Claude)` is called
- **Then** removes `.claude/commands/specsync/` entirely, but `.claude/commands/other-command.md` and `.claude/commands/` itself are untouched

### Scenario: Codex gets a skill only

- **Given** `install_agent(root, AgentTool::Codex)` is called
- **Then** `.codex/skills/spec-sync/SKILL.md` is created and no `.codex/commands/` directory is created at all

### Scenario: Upgrading spec-sync refreshes stale content

- **Given** `.claude/skills/spec-sync/SKILL.md` exists but contains content written by an older spec-sync version
- **When** `install_agent(root, AgentTool::Claude)` is called
- **Then** the file is overwritten with the current template content and `Ok(true)` is returned

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Cannot create skill/command directory | Returns `Err` with descriptive message |
| Cannot write skill/command file | Returns `Err` with descriptive message |
| Cannot remove skill directory or command file | Returns `Err` with descriptive message |

## Dependencies

### Consumes

| Crate/Module | What is used |
|-------------|-------------|
| colored | Terminal output formatting |

### Consumed By

| Module | What is used |
|--------|-------------|
| cmd_agents | `cmd_install`, `cmd_uninstall`, `cmd_status`, `AgentTool` |

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-07-01 | claude | v2: `install_agent` overwrites artifacts whose existing content differs from the current template (content-aware upgrade), instead of only writing missing files |
| 2026-07-01 | claude | Initial spec — native skill/command installation for Claude Code, Cursor, Codex, Gemini CLI |
