use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};

// ─── Shared instruction body ─────────────────────────────────────────────────
//
// This is a standalone copy of the workflow prose also found (in slightly
// different phrasing) in hooks.rs's CLAUDE_MD_SNIPPET/AGENTS_MD_SNIPPET. It is
// duplicated rather than shared because hooks.rs's four snippets are not
// identical to each other (Cursor/Copilot use a terser style) and unifying
// them is out of scope here — this body is purely for the new SKILL.md files.

const SKILL_BODY: &str = r#"## Companion files

## Verified SDD change lifecycle (5.0)

For every meaningful source, test, public documentation, schema, or configuration change:

1. Run `specsync change new "<intent>" --json` and conduct the returned interview with the user.
2. Use `specsync change answer <id> <question-id> <answer> --json` until no questions remain.
3. Complete the adaptively selected artifacts and semantic deltas. Requirements use stable
   `REQ-<module>-<number>` IDs, a normative SHALL statement, and acceptance criteria.
4. Ask the user for the definition approval, then run `specsync change approve <id>`.
5. Run `specsync change start <id>` before editing implementation code.
6. Keep tasks and artifacts current, then run `specsync change verify <id>`.
7. Present verification evidence and ask for closing approval. Only after explicit approval,
   run `specsync change accept <id>`; archive separately with `specsync change archive <id>`.

Never invent or self-grant either human approval. If an approved definition changes, its digest
becomes stale and must be approved again. `specsync check` validates canonical specs plus approved
active deltas, requirement-to-test evidence, change coverage, and CI gates.

Each canonical spec may have policy-selected companion files. Read and update the ones present; do not create empty companions only for ceremony:

- **`tasks.md`** — Work items for this module. Check off tasks (`- [x]`) as you complete them. Add new tasks if you discover work needed.
- **`requirements.md`** — Acceptance criteria and user stories. These are permanent invariants, not tasks — do not check them off. Update if requirements change.
- **`context.md`** — Architectural decisions, key files, and current status. Update when you make design decisions or change what's in progress.
- **`testing.md`** — Test strategy: automated test locations, manual QA checklists, and edge cases/boundary conditions.
- **`design.md`** *(opt-in)* — Layout, component hierarchy, design tokens, and asset references. Present when `companions.design` is enabled in config.

## Before modifying any module

1. Read the relevant spec in `specs/<module>/<module>.spec.md`
2. Read whichever companion files are present (`requirements.md`, `tasks.md`, `context.md`, `testing.md`, `design.md`, or project-defined files)
3. After changes, run `specsync check` to verify specs still pass

## After completing work

1. Mark completed items in `tasks.md` — check off finished tasks, add new ones discovered
2. Update `context.md` — record decisions made, update current status
3. If requirements changed, update `requirements.md` acceptance criteria
4. If test coverage changed, update `testing.md` with new test files or edge cases
5. If UI/layout changed, update `design.md` with revised layout, components, or tokens

## Before creating a PR

Run `specsync check --strict` — all specs must pass with zero warnings.

## When adding new modules

Run `specsync scaffold <module-name>` to create a spec, companion files, a registry
entry, and auto-detected source files — or `specsync new <module-name>` for a
minimal spec-only draft. Complete the spec before writing code. The
`/specsync:create-spec` command (or tool-equivalent) runs this for you, and
accepts either a bare module name or a natural-language feature description
(e.g. `/specsync:create-spec "I want a feature that lets users export their
data as CSV"`) — pass a description and it will pick a module name and use
the description to draft the spec's Purpose and Requirements.

## Key commands

- `specsync check` — validate all specs against source code
- `specsync check --json` — machine-readable validation output
- `specsync coverage` — show which modules lack specs
- `specsync score` — quality score for each spec (0-100)
- `specsync scaffold <name>` — full scaffold: spec + companions + registry entry + source detection
- `specsync new <name>` — quick-create a minimal spec (add `--full` for companions)
- `specsync resolve --remote` — verify cross-project dependencies
"#;

// ─── Create-spec command body (shared prose, per-tool argument syntax) ───────

const CREATE_SPEC_STEPS_MD: &str = r#"1. Parse the arguments above: the first whitespace-separated token is the
   module name. If the arguments also contain `--minimal` (in any position),
   remove it and remember that minimal mode was requested.
2. Look at whatever text remains. It will be one of:
   - **A bare module name** — a short identifier like `auth-service` or
     `billing`. Use it as-is.
   - **A free-text feature description** — a sentence or phrase describing
     what to build, e.g. `"I want a feature that lets users export their
     data as CSV"`. In this case, invent a short, kebab-case module name that
     captures the idea (e.g. `csv-export`). If the right name is ambiguous,
     ask the user to confirm or rename it before continuing. Keep the full
     description at hand — you'll use it in step 5.
3. If minimal mode was requested, run:
   ```
   specsync new <module-name>
   ```
   This creates a minimal spec only (no companion files).
4. Otherwise (default), run:
   ```
   specsync scaffold <module-name>
   ```
   This creates the spec, companion files (`tasks.md`, `requirements.md`,
   `context.md`, `testing.md`, and `design.md` if `companions.design` is
   enabled), a registry entry, and auto-detects related source files.
5. Open the newly created `specs/<module-name>/<module-name>.spec.md` and fill
   in the `Purpose`, `Requirements`, and `Public API` sections. If a free-text
   description was given in step 2, use it directly to draft these sections —
   ask clarifying questions if it's underspecified, but do not leave the
   sections as unfilled placeholder text. Do the same for `requirements.md`
   (acceptance criteria) and `tasks.md` (initial task breakdown), if present.
6. Run `specsync check` to confirm the new spec passes validation."#;

const CREATE_SPEC_STEPS_TOML: &str = r#"1. Parse the arguments above: the first whitespace-separated token is the
   module name. If the arguments also contain --minimal (in any position),
   remove it and remember that minimal mode was requested.
2. Look at whatever text remains. It will be one of:
   - A bare module name - a short identifier like auth-service or billing.
     Use it as-is.
   - A free-text feature description - a sentence or phrase describing what
     to build, e.g. "I want a feature that lets users export their data as
     CSV". In this case, invent a short, kebab-case module name that captures
     the idea (e.g. csv-export). If the right name is ambiguous, ask the user
     to confirm or rename it before continuing. Keep the full description at
     hand - you'll use it in step 5.
3. If minimal mode was requested, run:
   specsync new <module-name>
   This creates a minimal spec only (no companion files).
4. Otherwise (default), run:
   specsync scaffold <module-name>
   This creates the spec, companion files (tasks.md, requirements.md,
   context.md, testing.md, and design.md if companions.design is enabled),
   a registry entry, and auto-detects related source files.
5. Open the newly created specs/<module-name>/<module-name>.spec.md and fill
   in the Purpose, Requirements, and Public API sections. If a free-text
   description was given in step 2, use it directly to draft these sections -
   ask clarifying questions if it's underspecified, but do not leave the
   sections as unfilled placeholder text. Do the same for requirements.md
   (acceptance criteria) and tasks.md (initial task breakdown), if present.
6. Run specsync check to confirm the new spec passes validation."#;

const CREATE_SPEC_DESCRIPTION: &str = "Scaffold a new spec-sync module spec from a module name or a natural-language feature description (full scaffold by default, or minimal with --minimal)";

const CREATE_CHANGE_DESCRIPTION: &str =
    "Create and guide a verified spec-sync SDD change through its deterministic interview";

const CREATE_CHANGE_STEPS_MD: &str = r#"1. Run `specsync change new "$ARGUMENTS" --json`.
2. Read the returned `questions` array and interview the user one question at a time.
3. Record each answer with `specsync change answer <id> <question-id> <answer> --json`.
4. Continue until the question list is empty, then show the selected artifacts and next action.
5. Do not approve, implement, verify, accept, or archive until the corresponding human gate or work stage is reached."#;

const SKILL_TRIGGER_DESCRIPTION: &str = "Keep markdown module specs in specs/<module>/ synchronized with source code using spec-sync. Use this whenever creating, editing, or reviewing code in a module that has (or should have) a spec, or whenever the user mentions specs, spec-sync, companion files (tasks.md/requirements.md/context.md/testing.md/design.md), or asks to add/update a module's documentation.";

/// AI coding tools that receive native skill/command file installation, as
/// opposed to the prose-instruction-file hooks in `hooks.rs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTool {
    Claude,
    Cursor,
    Codex,
    Gemini,
}

impl AgentTool {
    pub fn all() -> &'static [AgentTool] {
        &[
            AgentTool::Claude,
            AgentTool::Cursor,
            AgentTool::Codex,
            AgentTool::Gemini,
        ]
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            AgentTool::Claude => "claude",
            AgentTool::Cursor => "cursor",
            AgentTool::Codex => "codex",
            AgentTool::Gemini => "gemini",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            AgentTool::Claude => "Claude Code SDD skill + specsync commands",
            AgentTool::Cursor => "Cursor SDD skill + specsync commands",
            AgentTool::Codex => "Codex CLI SDD skill (project-scoped)",
            AgentTool::Gemini => "Gemini CLI SDD skill + specsync commands",
        }
    }

    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude" => Some(AgentTool::Claude),
            "cursor" => Some(AgentTool::Cursor),
            "codex" => Some(AgentTool::Codex),
            "gemini" => Some(AgentTool::Gemini),
            _ => None,
        }
    }

    /// Directory (relative to project root) containing this tool's skill
    /// folder, e.g. `.claude/skills/spec-sync/`.
    fn skill_dir(&self, root: &Path) -> Option<PathBuf> {
        match self {
            AgentTool::Claude => Some(root.join(".claude").join("skills").join("spec-sync")),
            AgentTool::Cursor => Some(root.join(".cursor").join("skills").join("spec-sync")),
            AgentTool::Codex => Some(root.join(".codex").join("skills").join("spec-sync")),
            AgentTool::Gemini => Some(root.join(".gemini").join("skills").join("spec-sync")),
        }
    }

    /// Full path to this tool's `create-spec` command file. `None` if this
    /// tool has no project-local command mechanism (Codex).
    fn command_path(&self, root: &Path) -> Option<PathBuf> {
        match self {
            AgentTool::Claude => Some(
                root.join(".claude")
                    .join("commands")
                    .join("specsync")
                    .join("create-spec.md"),
            ),
            AgentTool::Cursor => Some(
                root.join(".cursor")
                    .join("commands")
                    .join("specsync-create-spec.md"),
            ),
            AgentTool::Gemini => Some(
                root.join(".gemini")
                    .join("commands")
                    .join("specsync")
                    .join("create-spec.toml"),
            ),
            AgentTool::Codex => None,
        }
    }

    fn change_command_path(&self, root: &Path) -> Option<PathBuf> {
        match self {
            AgentTool::Claude => Some(
                root.join(".claude")
                    .join("commands")
                    .join("specsync")
                    .join("create-change.md"),
            ),
            AgentTool::Cursor => Some(
                root.join(".cursor")
                    .join("commands")
                    .join("specsync-create-change.md"),
            ),
            AgentTool::Gemini => Some(
                root.join(".gemini")
                    .join("commands")
                    .join("specsync")
                    .join("create-change.toml"),
            ),
            AgentTool::Codex => None,
        }
    }
}

/// True if every artifact this tool should have (skill dir and/or command
/// file) already exists.
pub fn is_installed(root: &Path, tool: AgentTool) -> bool {
    let skill_ok = match tool.skill_dir(root) {
        Some(dir) => dir.join("SKILL.md").exists(),
        None => true,
    };
    let command_ok = [tool.command_path(root), tool.change_command_path(root)]
        .into_iter()
        .flatten()
        .all(|path| path.exists());
    skill_ok && command_ok
}

/// Install skill + command files for one tool. Returns Ok(true) if anything
/// was written, Ok(false) if everything was already present and up to date.
///
/// Re-running on an already-installed project upgrades stale content: if a
/// newer spec-sync ships revised skill/command text, the existing file is
/// overwritten to match rather than being silently left outdated because it
/// already exists. Safe because these files are fully spec-sync-owned (see
/// `uninstall_agent`'s doc comment) — there's no user content to preserve.
pub fn install_agent(root: &Path, tool: AgentTool) -> Result<bool, String> {
    let mut changed = false;

    if let Some(dir) = tool.skill_dir(root) {
        let skill_path = dir.join("SKILL.md");
        let skill_content = skill_md_content(tool);
        let needs_write = fs::read_to_string(&skill_path)
            .map(|existing| existing != skill_content)
            .unwrap_or(true);
        if needs_write {
            fs::create_dir_all(&dir)
                .map_err(|e| format!("Failed to create {}: {e}", dir.display()))?;
            fs::write(&skill_path, skill_content)
                .map_err(|e| format!("Failed to write {}: {e}", skill_path.display()))?;
            changed = true;
        }
    }

    if let Some(path) = tool.command_path(root) {
        let command_content = create_spec_command_content(tool);
        let needs_write = fs::read_to_string(&path)
            .map(|existing| existing != command_content)
            .unwrap_or(true);
        if needs_write {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create {}: {e}", parent.display()))?;
            }
            fs::write(&path, command_content)
                .map_err(|e| format!("Failed to write {}: {e}", path.display()))?;
            changed = true;
        }
    }

    if let Some(path) = tool.change_command_path(root) {
        let command_content = create_change_command_content(tool);
        let needs_write = fs::read_to_string(&path)
            .map(|existing| existing != command_content)
            .unwrap_or(true);
        if needs_write {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create {}: {e}", parent.display()))?;
            }
            fs::write(&path, command_content)
                .map_err(|e| format!("Failed to write {}: {e}", path.display()))?;
            changed = true;
        }
    }

    Ok(changed)
}

/// Uninstall a tool's skill dir and/or command file. Returns Ok(true) if
/// anything was removed, Ok(false) if nothing was present.
///
/// Safe by construction: these paths are dedicated files/directories that
/// spec-sync fully owns (a `spec-sync/`-named skill folder, a
/// `specsync`-namespaced command file), so removal never risks clobbering
/// unrelated user content the way editing a shared file like CLAUDE.md would.
pub fn uninstall_agent(root: &Path, tool: AgentTool) -> Result<bool, String> {
    let mut removed = false;

    if let Some(dir) = tool.skill_dir(root)
        && dir.exists()
    {
        fs::remove_dir_all(&dir).map_err(|e| format!("Failed to remove {}: {e}", dir.display()))?;
        removed = true;
    }

    for path in [tool.command_path(root), tool.change_command_path(root)]
        .into_iter()
        .flatten()
    {
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| format!("Failed to remove {}: {e}", path.display()))?;
            removed = true;
        }
        cleanup_command_namespace(&path);
    }

    Ok(removed)
}

// ─── Content builders ─────────────────────────────────────────────────────

fn skill_md_content(_tool: AgentTool) -> String {
    format!(
        "---\nname: spec-sync\ndescription: {SKILL_TRIGGER_DESCRIPTION}\n---\n\n# Spec-Sync Workflow\n\nThis project uses [spec-sync](https://github.com/CorvidLabs/spec-sync) for bidirectional spec-to-code validation. Specs live in `specs/<module>/<module>.spec.md`.\n\n{SKILL_BODY}",
    )
}

fn create_spec_command_content(tool: AgentTool) -> String {
    match tool {
        AgentTool::Claude => claude_create_spec_md(),
        AgentTool::Cursor => cursor_create_spec_md(),
        AgentTool::Gemini => gemini_create_spec_toml(),
        AgentTool::Codex => unreachable!("Codex has no command file"),
    }
}

fn create_change_command_content(tool: AgentTool) -> String {
    match tool {
        AgentTool::Claude => format!(
            "---\ndescription: {CREATE_CHANGE_DESCRIPTION}\nargument-hint: <change-description>\n---\n\n{CREATE_CHANGE_STEPS_MD}\n"
        ),
        AgentTool::Cursor => format!(
            "Create a verified spec-sync SDD change.\n\nArguments: $ARGUMENTS\n\n{CREATE_CHANGE_STEPS_MD}\n"
        ),
        AgentTool::Gemini => format!(
            "description = \"{CREATE_CHANGE_DESCRIPTION}\"\n\nprompt = \"\"\"\nArguments: {{{{args}}}}\n\n{CREATE_CHANGE_STEPS_MD}\n\"\"\"\n"
        ),
        AgentTool::Codex => unreachable!("Codex has no command file"),
    }
}

fn cleanup_command_namespace(path: &Path) {
    if let Some(parent) = path.parent() {
        let is_our_namespace_dir =
            parent.file_name().and_then(|name| name.to_str()) == Some("specsync");
        if is_our_namespace_dir
            && fs::read_dir(parent)
                .map(|mut entries| entries.next().is_none())
                .unwrap_or(false)
        {
            let _ = fs::remove_dir(parent);
        }
    }
}

fn claude_create_spec_md() -> String {
    format!(
        "---\ndescription: {CREATE_SPEC_DESCRIPTION}\nargument-hint: <module-name-or-description> [--minimal]\n---\n\nCreate a new spec-sync module spec.\n\nArguments: `$ARGUMENTS`\n\n{CREATE_SPEC_STEPS_MD}\n",
    )
}

fn cursor_create_spec_md() -> String {
    format!(
        "Create a new spec-sync module spec.\n\nArguments: $ARGUMENTS\n\n{CREATE_SPEC_STEPS_MD}\n",
    )
}

fn gemini_create_spec_toml() -> String {
    format!(
        "description = \"{CREATE_SPEC_DESCRIPTION}\"\n\nprompt = \"\"\"\nCreate a new spec-sync module spec.\n\nArguments: {{{{args}}}}\n\n{CREATE_SPEC_STEPS_TOML}\n\"\"\"\n",
    )
}

// ─── CLI command handlers ────────────────────────────────────────────────────

/// Install agent integrations for the specified tools (or all if empty).
pub fn cmd_install(root: &Path, targets: &[AgentTool]) {
    let targets = if targets.is_empty() {
        AgentTool::all().to_vec()
    } else {
        targets.to_vec()
    };

    println!(
        "\n--- {} ------------------------------------------------",
        "Installing Agent Integrations".bold()
    );

    let mut installed = 0;
    let mut skipped = 0;
    let mut errors = 0;

    for target in &targets {
        match install_agent(root, *target) {
            Ok(true) => {
                println!("  {} Installed {}", "✓".green(), target.description());
                installed += 1;
            }
            Ok(false) => {
                println!(
                    "  {} Already installed: {}",
                    "·".dimmed(),
                    target.description()
                );
                skipped += 1;
            }
            Err(e) => {
                println!("  {} {}: {e}", "✗".red(), target.description());
                errors += 1;
            }
        }
    }

    println!();
    if installed > 0 {
        println!("{installed} tool(s) installed.");
    }
    if skipped > 0 {
        println!("{skipped} tool(s) already present.");
    }
    if errors > 0 {
        println!("{errors} tool(s) failed.");
        std::process::exit(1);
    }
}

/// Uninstall agent integrations for the specified tools (or all if empty).
pub fn cmd_uninstall(root: &Path, targets: &[AgentTool]) {
    let targets = if targets.is_empty() {
        AgentTool::all().to_vec()
    } else {
        targets.to_vec()
    };

    println!(
        "\n--- {} ------------------------------------------------",
        "Uninstalling Agent Integrations".bold()
    );

    let mut removed = 0;

    for target in &targets {
        match uninstall_agent(root, *target) {
            Ok(true) => {
                println!("  {} Removed {}", "✓".green(), target.description());
                removed += 1;
            }
            Ok(false) => {
                println!("  {} Not installed: {}", "·".dimmed(), target.description());
            }
            Err(e) => {
                println!("  {} {}: {e}", "!".yellow(), target.description());
            }
        }
    }

    println!();
    if removed > 0 {
        println!("{removed} tool(s) removed.");
    } else {
        println!("No agent integrations to remove.");
    }
}

/// Show installation status of all agent tools.
pub fn cmd_status(root: &Path) {
    println!(
        "\n--- {} ------------------------------------------------",
        "Agent Integration Status".bold()
    );

    for target in AgentTool::all() {
        let installed = is_installed(root, *target);
        let status = if installed {
            "installed".green().to_string()
        } else {
            "not installed".dimmed().to_string()
        };
        println!("  {:45} {}", target.description(), status);
    }

    println!();
    println!("Install all: specsync agents install");
    println!("Install one: specsync agents install --claude --gemini");
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    // ── AgentTool::all / name / from_str ───────────────────────────

    #[test]
    fn agent_tool_all_returns_four_targets() {
        assert_eq!(AgentTool::all().len(), 4);
    }

    #[test]
    fn agent_tool_all_contains_all_variants() {
        let all = AgentTool::all();
        assert!(all.contains(&AgentTool::Claude));
        assert!(all.contains(&AgentTool::Cursor));
        assert!(all.contains(&AgentTool::Codex));
        assert!(all.contains(&AgentTool::Gemini));
    }

    #[test]
    fn agent_tool_name_returns_expected_strings() {
        assert_eq!(AgentTool::Claude.name(), "claude");
        assert_eq!(AgentTool::Cursor.name(), "cursor");
        assert_eq!(AgentTool::Codex.name(), "codex");
        assert_eq!(AgentTool::Gemini.name(), "gemini");
    }

    #[test]
    fn from_str_parses_all_targets() {
        assert_eq!(AgentTool::from_str("claude"), Some(AgentTool::Claude));
        assert_eq!(AgentTool::from_str("cursor"), Some(AgentTool::Cursor));
        assert_eq!(AgentTool::from_str("codex"), Some(AgentTool::Codex));
        assert_eq!(AgentTool::from_str("gemini"), Some(AgentTool::Gemini));
    }

    #[test]
    fn from_str_is_case_insensitive() {
        assert_eq!(AgentTool::from_str("CLAUDE"), Some(AgentTool::Claude));
        assert_eq!(AgentTool::from_str("Gemini"), Some(AgentTool::Gemini));
    }

    #[test]
    fn from_str_returns_none_for_unknown() {
        assert_eq!(AgentTool::from_str("unknown"), None);
        assert_eq!(AgentTool::from_str(""), None);
        assert_eq!(AgentTool::from_str("windsurf"), None);
    }

    // ── path correctness ────────────────────────────────────────────

    #[test]
    fn claude_paths_are_correct() {
        let tmp = setup();
        assert_eq!(
            AgentTool::Claude.skill_dir(tmp.path()).unwrap(),
            tmp.path().join(".claude/skills/spec-sync")
        );
        assert_eq!(
            AgentTool::Claude.command_path(tmp.path()).unwrap(),
            tmp.path().join(".claude/commands/specsync/create-spec.md")
        );
    }

    #[test]
    fn cursor_command_path_is_flat() {
        let tmp = setup();
        assert_eq!(
            AgentTool::Cursor.command_path(tmp.path()).unwrap(),
            tmp.path().join(".cursor/commands/specsync-create-spec.md")
        );
    }

    #[test]
    fn codex_has_no_command_path() {
        let tmp = setup();
        assert!(AgentTool::Codex.command_path(tmp.path()).is_none());
        assert!(AgentTool::Codex.skill_dir(tmp.path()).is_some());
    }

    #[test]
    fn gemini_has_both_skill_and_command_paths() {
        let tmp = setup();
        assert!(AgentTool::Gemini.skill_dir(tmp.path()).is_some());
        assert!(AgentTool::Gemini.command_path(tmp.path()).is_some());
    }

    // ── is_installed ───────────────────────────────────────────────

    #[test]
    fn is_installed_returns_false_for_empty_dir() {
        let tmp = setup();
        for target in AgentTool::all() {
            assert!(!is_installed(tmp.path(), *target));
        }
    }

    // ── install_agent ───────────────────────────────────────────────

    #[test]
    fn install_claude_creates_skill_and_command() {
        let tmp = setup();
        assert!(install_agent(tmp.path(), AgentTool::Claude).unwrap());

        let skill =
            fs::read_to_string(tmp.path().join(".claude/skills/spec-sync/SKILL.md")).unwrap();
        assert!(skill.starts_with("---\nname: spec-sync"));
        assert!(skill.contains("tasks.md"));
        assert!(skill.contains("specsync check"));

        let command =
            fs::read_to_string(tmp.path().join(".claude/commands/specsync/create-spec.md"))
                .unwrap();
        assert!(command.starts_with("---\ndescription:"));
        assert!(command.contains("$ARGUMENTS"));
        assert!(command.contains("specsync scaffold"));
        assert!(command.contains("specsync new"));
        let change_command = fs::read_to_string(
            tmp.path()
                .join(".claude/commands/specsync/create-change.md"),
        )
        .unwrap();
        assert!(change_command.contains("specsync change new"));

        assert!(is_installed(tmp.path(), AgentTool::Claude));
    }

    #[test]
    fn install_cursor_command_has_no_frontmatter() {
        let tmp = setup();
        assert!(install_agent(tmp.path(), AgentTool::Cursor).unwrap());
        let command =
            fs::read_to_string(tmp.path().join(".cursor/commands/specsync-create-spec.md"))
                .unwrap();
        assert!(!command.starts_with("---"));
        assert!(command.contains("$ARGUMENTS"));
        assert!(
            tmp.path()
                .join(".cursor/commands/specsync-create-change.md")
                .exists()
        );
    }

    #[test]
    fn install_codex_creates_skill_only() {
        let tmp = setup();
        assert!(install_agent(tmp.path(), AgentTool::Codex).unwrap());
        assert!(tmp.path().join(".codex/skills/spec-sync/SKILL.md").exists());
        assert!(!tmp.path().join(".codex/commands").exists());
        assert!(is_installed(tmp.path(), AgentTool::Codex));
    }

    #[test]
    fn install_gemini_creates_skill_and_command() {
        let tmp = setup();
        assert!(install_agent(tmp.path(), AgentTool::Gemini).unwrap());
        assert!(
            tmp.path()
                .join(".gemini/skills/spec-sync/SKILL.md")
                .exists()
        );

        let path = tmp
            .path()
            .join(".gemini/commands/specsync/create-spec.toml");
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("prompt = \"\"\""));
        assert!(content.contains("description ="));
        assert!(content.contains("{{args}}"));
        // Triple-quoted block must be balanced.
        assert_eq!(content.matches("\"\"\"").count(), 2);

        assert!(is_installed(tmp.path(), AgentTool::Gemini));
    }

    #[test]
    fn install_is_idempotent() {
        let tmp = setup();
        for target in AgentTool::all() {
            assert!(install_agent(tmp.path(), *target).unwrap());
            assert!(!install_agent(tmp.path(), *target).unwrap());
        }
    }

    #[test]
    fn install_overwrites_stale_skill_content() {
        let tmp = setup();
        let skill_path = tmp.path().join(".claude/skills/spec-sync/SKILL.md");
        fs::create_dir_all(skill_path.parent().unwrap()).unwrap();
        fs::write(&skill_path, "stale content from an older spec-sync version").unwrap();

        assert!(install_agent(tmp.path(), AgentTool::Claude).unwrap());

        let content = fs::read_to_string(&skill_path).unwrap();
        assert!(content.starts_with("---\nname: spec-sync"));
        assert!(!content.contains("stale content"));
    }

    #[test]
    fn install_overwrites_stale_command_content() {
        let tmp = setup();
        let command_path = tmp.path().join(".claude/commands/specsync/create-spec.md");
        fs::create_dir_all(command_path.parent().unwrap()).unwrap();
        fs::write(&command_path, "stale command body").unwrap();

        assert!(install_agent(tmp.path(), AgentTool::Claude).unwrap());

        let content = fs::read_to_string(&command_path).unwrap();
        assert!(content.contains("$ARGUMENTS"));
        assert!(!content.contains("stale command body"));
    }

    #[test]
    fn install_does_not_rewrite_unchanged_content() {
        let tmp = setup();
        install_agent(tmp.path(), AgentTool::Claude).unwrap();
        // Second install: content is identical, so nothing should be rewritten.
        assert!(!install_agent(tmp.path(), AgentTool::Claude).unwrap());
    }

    // ── uninstall_agent ──────────────────────────────────────────────

    #[test]
    fn uninstall_returns_false_when_not_installed() {
        let tmp = setup();
        for target in AgentTool::all() {
            assert!(!uninstall_agent(tmp.path(), *target).unwrap());
        }
    }

    #[test]
    fn uninstall_claude_removes_skill_and_command() {
        let tmp = setup();
        install_agent(tmp.path(), AgentTool::Claude).unwrap();
        assert!(uninstall_agent(tmp.path(), AgentTool::Claude).unwrap());
        assert!(!tmp.path().join(".claude/skills/spec-sync").exists());
        assert!(!tmp.path().join(".claude/commands/specsync").exists());
        assert!(!is_installed(tmp.path(), AgentTool::Claude));
    }

    #[test]
    fn uninstall_preserves_sibling_commands() {
        let tmp = setup();
        install_agent(tmp.path(), AgentTool::Claude).unwrap();
        let sibling = tmp.path().join(".claude/commands/other-command.md");
        fs::write(&sibling, "unrelated command").unwrap();

        assert!(uninstall_agent(tmp.path(), AgentTool::Claude).unwrap());

        assert!(!tmp.path().join(".claude/commands/specsync").exists());
        assert!(tmp.path().join(".claude/commands").exists());
        assert!(sibling.exists());
    }

    #[test]
    fn uninstall_cursor_flat_file_does_not_touch_commands_dir() {
        let tmp = setup();
        install_agent(tmp.path(), AgentTool::Cursor).unwrap();
        assert!(uninstall_agent(tmp.path(), AgentTool::Cursor).unwrap());
        assert!(
            !tmp.path()
                .join(".cursor/commands/specsync-create-spec.md")
                .exists()
        );
        assert!(tmp.path().join(".cursor/commands").exists());
    }

    #[test]
    fn uninstall_gemini_removes_skill_and_command() {
        let tmp = setup();
        install_agent(tmp.path(), AgentTool::Gemini).unwrap();
        assert!(uninstall_agent(tmp.path(), AgentTool::Gemini).unwrap());
        assert!(!tmp.path().join(".gemini/commands/specsync").exists());
        assert!(!tmp.path().join(".gemini/skills/spec-sync").exists());
    }

    #[test]
    fn uninstall_codex_removes_skill_only() {
        let tmp = setup();
        install_agent(tmp.path(), AgentTool::Codex).unwrap();
        assert!(uninstall_agent(tmp.path(), AgentTool::Codex).unwrap());
        assert!(!tmp.path().join(".codex/skills/spec-sync").exists());
    }
}
