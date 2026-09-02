---
module: agents
version: 14
status: stable
files:
  - src/agents.rs
db_tables: []
depends_on: []
---

# Agents

## Purpose

Installs native, tool-owned verified-SDD skills for Claude Code, Cursor, Codex, and Gemini CLI.
Generated artifacts are tracked in `.specsync/agent-artifacts.json` so upgrades and uninstall can
distinguish exact managed bytes from user customization.

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

1. Each `AgentTool` owns an SDD skill and, where supported, both `create-spec` and `create-change` commands — Codex has the project skill only because its command mechanism is deprecated/global.
2. Installation is idempotent per artifact. It updates only missing files, exact bytes recorded in
   the versioned manifest, or legacy bytes matching a known generated template.
3. A customized, untracked, non-UTF-8, or digest-mismatched artifact is preserved and reported as
   an actionable conflict; one conflict prevents all writes for that tool.
4. Uninstall removes only digest-matching managed files, aggregates conflicts before mutation,
   prunes matching manifest entries, and removes a managed directory only when it is empty.
5. Cursor's command file is flat (`.cursor/commands/specsync-create-spec.md`, no namespaced subdirectory, no YAML frontmatter) since Cursor's command mechanism doesn't support either.
6. Claude and Gemini's commands live in a namespaced subdirectory (`.claude/commands/specsync/create-spec.md`, `.gemini/commands/specsync/create-spec.toml`) so they're invoked as `/specsync:create-spec`.
7. Gemini's command file is TOML (`description`/`prompt` keys, `{{args}}` placeholder), hand-built as a string template since no `toml` crate dependency exists in this project.
8. Empty targets list means "all tools", matching the `hooks` module's convention.
9. `cmd_install` exits with code 1 if any tool installation fails.
10. Generated create-spec and create-change assets preserve complete arguments using each tool's native placeholder and quote free-text interview answers as one CLI argument.
11. Repository-owned native create-spec commands are exact installer outputs, strip supported flags before complete-input classification, and cannot silently drift from their shared templates.

## Behavioral Examples

### Scenario: Install all agent tools

- **Given** a project with no agent integrations installed
- **When** `cmd_install(root, &[])` is called
- **Then** installs four skills plus create-spec and create-change commands for Claude, Cursor, and Gemini

### Scenario: Already installed

- **Given** Claude's skill, create-spec command, and create-change command all exist with current content
- **When** `install_agent(root, AgentTool::Claude)` is called
- **Then** returns `Ok(false)` without modifying any artifact

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
| Generated artifact was customized | Preserves every customized artifact and returns one actionable conflict report |

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
| 2026-08-30 | SpecSync | v12: generated `change check` skill describes spec↔code sync, not targeted tests |
| 2026-07-10 | codex | v3: teach all four skills the verified SDD lifecycle and add create-change commands where supported |
| 2026-07-01 | claude | v2: `install_agent` overwrites artifacts whose existing content differs from the current template (content-aware upgrade), instead of only writing missing files |
| 2026-07-01 | claude | Initial spec — native skill/command installation for Claude Code, Cursor, Codex, Gemini CLI |
| 2026-07-11 | CHG-0003-finalize-specsync-5-0-release-consistency-and-parallel-validation: Finalize SpecSync 5.0 release consistency and parallel validation |
| 2026-07-13 | CHG-0016-preserve-free-text-arguments-in-generated-agent-commands: Preserve free-text arguments in generated agent commands |
| 2026-07-15 | SpecSync | CHG-0041-synchronize-generated-create-spec-agent-commands-with-the-corrected-free-text-pa: Synchronize generated create-spec agent commands with the corrected free-text parser guidance and prevent checked-in asset drift |
| 2026-07-30 | SpecSync | CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara: Stabilize SpecSync 6.0 with one scope approval, same-PR finalization, lightweight archive CI, scoped review, and selected UX fixes |
| 2026-07-31 | SpecSync | CHG-0069-scoped-change-check-change-audit-and-agent-pack-for-the-two-verb-lifecycle: Scoped change check, change audit, and agent pack for the two-verb lifecycle |
| 2026-07-31 | SpecSync | CHG-0070-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes: Land pre-6.0 product fixes for hooks init coverage naming and exit codes |
| 2026-08-01 | SpecSync | CHG-0071-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes-scoped: Land pre-6.0 product fixes for hooks init coverage naming and exit codes (scoped paths) |
| 2026-08-19 | SpecSync | CHG-0158-the-forward-compatibility-valve-must-be-true-everywhere-it-is-claimed: The forward-compatibility valve must be true everywhere it is claimed |
| 2026-08-30 | SpecSync | make-check-the-product-and-stop-change-check-from-spawning-project-tests: Make check the product and stop change check from spawning project tests |
| 2026-09-02 | SpecSync | tell-agents-when-it-is-safe-to-clear-context: Tell agents when it is safe to clear context |
