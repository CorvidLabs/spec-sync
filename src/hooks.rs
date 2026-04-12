use colored::Colorize;
use std::fs;
use std::path::Path;

// ─── Agent instruction templates ─────────────────────────────────────────────

const CLAUDE_MD_SNIPPET: &str = r#"# Spec-Sync Integration

This project uses [spec-sync](https://github.com/CorvidLabs/spec-sync) for bidirectional spec-to-code validation.

## Companion files

Each spec in `specs/<module>/` has companion files — read them before working, update them after:

- **`tasks.md`** — Work items for this module. Check off tasks (`- [x]`) as you complete them. Add new tasks if you discover work needed.
- **`requirements.md`** — Acceptance criteria and user stories. These are permanent invariants, not tasks — do not check them off. Update if requirements change.
- **`context.md`** — Architectural decisions, key files, and current status. Update when you make design decisions or change what's in progress.
- **`testing.md`** — Test strategy: automated test locations, manual QA checklists, and edge cases/boundary conditions.
- **`design.md`** *(opt-in)* — Layout, component hierarchy, design tokens, and asset references. Present when `companions.design` is enabled in config.

## Before modifying any module

1. Read the relevant spec in `specs/<module>/<module>.spec.md`
2. Read companion files: `tasks.md`, `requirements.md`, `context.md`, `testing.md`, and `design.md` (if present)
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

Run `specsync add-spec <module-name>` to scaffold the spec and companion files, then fill in the spec before writing code.

## Key commands

- `specsync check` — validate all specs against source code
- `specsync check --json` — machine-readable validation output
- `specsync coverage` — show which modules lack specs
- `specsync score` — quality score for each spec (0-100)
- `specsync add-spec <name>` — scaffold a new spec with companion files
- `specsync resolve --remote` — verify cross-project dependencies
"#;

const CURSORRULES_SNIPPET: &str = r#"# Spec-Sync Rules

This project uses spec-sync for spec-to-code validation. Specs live in the `specs/` directory.

## Companion files

Each spec directory has companion files — read before working, update after:

- `tasks.md` — Work items. Check off completed tasks, add new ones discovered.
- `requirements.md` — Acceptance criteria and user stories. Permanent invariants, not tasks.
- `context.md` — Decisions, key files, current status. Update when you make design choices.
- `testing.md` — Test strategy: automated test locations, manual QA checklists, edge cases.
- `design.md` *(opt-in)* — Layout, component hierarchy, design tokens, and asset references.

## Rules

- Before editing a module, read its spec at `specs/<module>/<module>.spec.md`
- Read `tasks.md`, `requirements.md`, `context.md`, `testing.md`, and `design.md` (if present) for outstanding work, requirements, decisions, test strategy, and design specs
- After modifying code, ensure `specsync check` still passes
- After completing work, update `tasks.md` (check off done items) and `context.md` (record decisions, update status)
- When creating new modules, run `specsync add-spec <module-name>` first
- Keep specs in sync: if you change exports, parameters, or types, update the spec's Public API table
- Run `specsync check --strict` before committing
"#;

const COPILOT_INSTRUCTIONS_SNIPPET: &str = r#"# Spec-Sync Integration

This project uses spec-sync for bidirectional spec-to-code validation.

## Companion files

Each spec directory has companion files — read before working, update after:

- `tasks.md` — Work items to check off as completed. Add new tasks if you discover work needed.
- `requirements.md` — Acceptance criteria and user stories. Permanent invariants, not checkable tasks.
- `context.md` — Architectural decisions, key files, and current status. Update with decisions made.
- `testing.md` — Test strategy: automated test locations, manual QA checklists, edge cases.
- `design.md` *(opt-in)* — Layout, component hierarchy, design tokens, and asset references.

## Guidelines

- Specs are in `specs/<module>/<module>.spec.md` — read the relevant spec before modifying a module
- Read companion files `tasks.md`, `requirements.md`, `context.md`, `testing.md`, and `design.md` (if present) before starting work
- After changes, `specsync check` should pass with no errors
- After completing work, update `tasks.md` (mark done items) and `context.md` (record decisions, update status)
- New modules need specs: run `specsync add-spec <module-name>`
- Keep the Public API table in each spec up to date with actual exports
"#;

const AGENTS_MD_SNIPPET: &str = r#"# Spec-Sync Integration

This project uses [spec-sync](https://github.com/CorvidLabs/spec-sync) for bidirectional spec-to-code validation.

## Companion files

Each spec in `specs/<module>/` has companion files — read them before working, update them after:

- **`tasks.md`** — Work items for this module. Check off tasks (`- [x]`) as you complete them. Add new tasks if you discover work needed.
- **`requirements.md`** — Acceptance criteria and user stories. These are permanent invariants, not tasks — do not check them off. Update if requirements change.
- **`context.md`** — Architectural decisions, key files, and current status. Update when you make design decisions or change what's in progress.
- **`testing.md`** — Test strategy: automated test locations, manual QA checklists, and edge cases/boundary conditions.
- **`design.md`** *(opt-in)* — Layout, component hierarchy, design tokens, and asset references. Present when `companions.design` is enabled in config.

## Before modifying any module

1. Read the relevant spec in `specs/<module>/<module>.spec.md`
2. Read companion files: `tasks.md`, `requirements.md`, `context.md`, `testing.md`, and `design.md` (if present)
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

Run `specsync add-spec <module-name>` to scaffold the spec and companion files, then fill in the spec before writing code.

## Key commands

- `specsync check` — validate all specs against source code
- `specsync check --json` — machine-readable validation output
- `specsync coverage` — show which modules lack specs
- `specsync score` — quality score for each spec (0-100)
- `specsync add-spec <name>` — scaffold a new spec with companion files
- `specsync resolve --remote` — verify cross-project dependencies
"#;

const PRE_COMMIT_HOOK: &str = r#"#!/bin/sh
# spec-sync pre-commit hook — validates specs before allowing commits.
# Installed by: specsync hooks install --precommit
# Remove by deleting this file or running: specsync hooks uninstall --precommit
#
# Enforcement is controlled by the `enforcement` field in specsync.json:
#   "warn"         — report violations but never block commits (default)
#   "enforce-new"  — block commits if files without specs exist
#   "strict"       — block commits on any validation error
# Override with --enforcement <mode> below if needed.

if command -v specsync >/dev/null 2>&1; then
    echo "specsync: checking specs..."
    if ! specsync check; then
        echo ""
        echo "specsync: specs have errors — fix them before committing."
        echo "  Run 'specsync check' to see details."
        echo "  Use 'git commit --no-verify' to skip this check."
        exit 1
    fi
else
    echo "specsync: not installed, skipping spec check"
fi
"#;

const CLAUDE_CODE_HOOK_SETTINGS: &str = r#"{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Edit|Write|NotebookEdit",
        "hooks": [
          {
            "type": "command",
            "command": "specsync check --json 2>/dev/null | head -1 || true"
          }
        ]
      }
    ]
  }
}"#;

/// All hook targets that can be installed.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum HookTarget {
    Claude,
    Cursor,
    Copilot,
    Agents,
    Precommit,
    ClaudeCodeHook,
}

impl HookTarget {
    pub fn all() -> &'static [HookTarget] {
        &[
            HookTarget::Claude,
            HookTarget::Cursor,
            HookTarget::Copilot,
            HookTarget::Agents,
            HookTarget::Precommit,
            HookTarget::ClaudeCodeHook,
        ]
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            HookTarget::Claude => "claude",
            HookTarget::Cursor => "cursor",
            HookTarget::Copilot => "copilot",
            HookTarget::Agents => "agents",
            HookTarget::Precommit => "precommit",
            HookTarget::ClaudeCodeHook => "claude-code-hook",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            HookTarget::Claude => "CLAUDE.md agent instructions",
            HookTarget::Cursor => ".cursorrules agent instructions",
            HookTarget::Copilot => ".github/copilot-instructions.md",
            HookTarget::Agents => "AGENTS.md agent instructions",
            HookTarget::Precommit => "Git pre-commit hook",
            HookTarget::ClaudeCodeHook => "Claude Code settings.json hook",
        }
    }

    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude" => Some(HookTarget::Claude),
            "cursor" => Some(HookTarget::Cursor),
            "copilot" => Some(HookTarget::Copilot),
            "agents" => Some(HookTarget::Agents),
            "precommit" | "pre-commit" => Some(HookTarget::Precommit),
            "claude-code-hook" | "claude-hook" => Some(HookTarget::ClaudeCodeHook),
            _ => None,
        }
    }
}

/// Check if a hook target is already installed.
pub fn is_installed(root: &Path, target: HookTarget) -> bool {
    match target {
        HookTarget::Claude => {
            let path = root.join("CLAUDE.md");
            path.exists()
                && fs::read_to_string(&path)
                    .map(|c| c.contains("Spec-Sync Integration"))
                    .unwrap_or(false)
        }
        HookTarget::Cursor => {
            let path = root.join(".cursorrules");
            path.exists()
                && fs::read_to_string(&path)
                    .map(|c| c.contains("Spec-Sync Rules"))
                    .unwrap_or(false)
        }
        HookTarget::Copilot => {
            let path = root.join(".github").join("copilot-instructions.md");
            path.exists()
                && fs::read_to_string(&path)
                    .map(|c| c.contains("Spec-Sync Integration"))
                    .unwrap_or(false)
        }
        HookTarget::Agents => {
            let path = root.join("AGENTS.md");
            path.exists()
                && fs::read_to_string(&path)
                    .map(|c| c.contains("Spec-Sync Integration"))
                    .unwrap_or(false)
        }
        HookTarget::Precommit => {
            let path = root.join(".git").join("hooks").join("pre-commit");
            path.exists()
                && fs::read_to_string(&path)
                    .map(|c| c.contains("spec-sync pre-commit hook"))
                    .unwrap_or(false)
        }
        HookTarget::ClaudeCodeHook => {
            let path = root.join(".claude").join("settings.json");
            path.exists()
                && fs::read_to_string(&path)
                    .map(|c| c.contains("specsync check"))
                    .unwrap_or(false)
        }
    }
}

/// Install a single hook target. Returns Ok(true) if installed, Ok(false) if already present.
pub fn install_hook(root: &Path, target: HookTarget) -> Result<bool, String> {
    if is_installed(root, target) {
        return Ok(false);
    }

    match target {
        HookTarget::Claude => install_claude_md(root),
        HookTarget::Cursor => install_cursorrules(root),
        HookTarget::Copilot => install_copilot(root),
        HookTarget::Agents => install_agents_md(root),
        HookTarget::Precommit => install_precommit(root),
        HookTarget::ClaudeCodeHook => install_claude_code_hook(root),
    }
}

/// Uninstall a single hook target. Returns Ok(true) if removed, Ok(false) if not found.
pub fn uninstall_hook(root: &Path, target: HookTarget) -> Result<bool, String> {
    if !is_installed(root, target) {
        return Ok(false);
    }

    match target {
        HookTarget::Claude => {
            // Remove the spec-sync section from CLAUDE.md
            let path = root.join("CLAUDE.md");
            remove_section_from_file(&path, "# Spec-Sync Integration")
        }
        HookTarget::Cursor => {
            let path = root.join(".cursorrules");
            remove_section_from_file(&path, "# Spec-Sync Rules")
        }
        HookTarget::Copilot => {
            let path = root.join(".github").join("copilot-instructions.md");
            remove_section_from_file(&path, "# Spec-Sync Integration")
        }
        HookTarget::Agents => {
            let path = root.join("AGENTS.md");
            remove_section_from_file(&path, "# Spec-Sync Integration")
        }
        HookTarget::Precommit => {
            let path = root.join(".git").join("hooks").join("pre-commit");
            if path.exists() {
                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read pre-commit hook: {e}"))?;
                if content.contains("spec-sync pre-commit hook") {
                    // If the entire file is our hook, remove it
                    if content.trim().starts_with("#!/bin/sh")
                        && content.contains("specsync check")
                        && content.lines().count() < 35
                    {
                        fs::remove_file(&path)
                            .map_err(|e| format!("Failed to remove pre-commit hook: {e}"))?;
                        return Ok(true);
                    }
                }
            }
            Ok(false)
        }
        HookTarget::ClaudeCodeHook => {
            // Don't auto-remove Claude Code settings — too risky
            Err(
                "Claude Code hook settings must be removed manually from .claude/settings.json"
                    .to_string(),
            )
        }
    }
}

// ─── Installation helpers ────────────────────────────────────────────────────

fn install_claude_md(root: &Path) -> Result<bool, String> {
    let path = root.join("CLAUDE.md");

    if path.exists() {
        // Append to existing CLAUDE.md
        let existing =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read CLAUDE.md: {e}"))?;

        if existing.contains("Spec-Sync") {
            return Ok(false);
        }

        let new_content = format!("{}\n\n{}", existing.trim_end(), CLAUDE_MD_SNIPPET);
        fs::write(&path, new_content).map_err(|e| format!("Failed to write CLAUDE.md: {e}"))?;
    } else {
        fs::write(&path, CLAUDE_MD_SNIPPET)
            .map_err(|e| format!("Failed to create CLAUDE.md: {e}"))?;
    }

    Ok(true)
}

fn install_cursorrules(root: &Path) -> Result<bool, String> {
    let path = root.join(".cursorrules");

    if path.exists() {
        let existing =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read .cursorrules: {e}"))?;

        if existing.contains("Spec-Sync") {
            return Ok(false);
        }

        let new_content = format!("{}\n\n{}", existing.trim_end(), CURSORRULES_SNIPPET);
        fs::write(&path, new_content).map_err(|e| format!("Failed to write .cursorrules: {e}"))?;
    } else {
        fs::write(&path, CURSORRULES_SNIPPET)
            .map_err(|e| format!("Failed to create .cursorrules: {e}"))?;
    }

    Ok(true)
}

fn install_copilot(root: &Path) -> Result<bool, String> {
    let github_dir = root.join(".github");
    if !github_dir.exists() {
        fs::create_dir_all(&github_dir).map_err(|e| format!("Failed to create .github/: {e}"))?;
    }

    let path = github_dir.join("copilot-instructions.md");

    if path.exists() {
        let existing = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read copilot-instructions.md: {e}"))?;

        if existing.contains("Spec-Sync") {
            return Ok(false);
        }

        let new_content = format!(
            "{}\n\n{}",
            existing.trim_end(),
            COPILOT_INSTRUCTIONS_SNIPPET
        );
        fs::write(&path, new_content)
            .map_err(|e| format!("Failed to write copilot-instructions.md: {e}"))?;
    } else {
        fs::write(&path, COPILOT_INSTRUCTIONS_SNIPPET)
            .map_err(|e| format!("Failed to create copilot-instructions.md: {e}"))?;
    }

    Ok(true)
}

fn install_agents_md(root: &Path) -> Result<bool, String> {
    let path = root.join("AGENTS.md");

    if path.exists() {
        let existing =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read AGENTS.md: {e}"))?;

        if existing.contains("Spec-Sync") {
            return Ok(false);
        }

        let new_content = format!("{}\n\n{}", existing.trim_end(), AGENTS_MD_SNIPPET);
        fs::write(&path, new_content).map_err(|e| format!("Failed to write AGENTS.md: {e}"))?;
    } else {
        fs::write(&path, AGENTS_MD_SNIPPET)
            .map_err(|e| format!("Failed to create AGENTS.md: {e}"))?;
    }

    Ok(true)
}

fn install_precommit(root: &Path) -> Result<bool, String> {
    let hooks_dir = root.join(".git").join("hooks");
    if !hooks_dir.exists() {
        fs::create_dir_all(&hooks_dir).map_err(|e| format!("Failed to create .git/hooks/: {e}"))?;
    }

    let path = hooks_dir.join("pre-commit");

    if path.exists() {
        let existing = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read pre-commit hook: {e}"))?;

        if existing.contains("specsync") {
            return Ok(false);
        }

        // Append to existing pre-commit hook
        let new_content = format!(
            "{}\n\n# --- spec-sync pre-commit hook ---\n{}",
            existing.trim_end(),
            PRE_COMMIT_HOOK
                .lines()
                .skip(1) // Skip the shebang since the existing file has one
                .collect::<Vec<_>>()
                .join("\n")
        );
        fs::write(&path, new_content)
            .map_err(|e| format!("Failed to write pre-commit hook: {e}"))?;
    } else {
        fs::write(&path, PRE_COMMIT_HOOK)
            .map_err(|e| format!("Failed to create pre-commit hook: {e}"))?;
    }

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        fs::set_permissions(&path, perms)
            .map_err(|e| format!("Failed to set pre-commit hook permissions: {e}"))?;
    }

    Ok(true)
}

fn install_claude_code_hook(root: &Path) -> Result<bool, String> {
    let claude_dir = root.join(".claude");
    if !claude_dir.exists() {
        fs::create_dir_all(&claude_dir).map_err(|e| format!("Failed to create .claude/: {e}"))?;
    }

    let path = claude_dir.join("settings.json");

    if path.exists() {
        let existing = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read .claude/settings.json: {e}"))?;

        if existing.contains("specsync") {
            return Ok(false);
        }

        // Parse existing JSON, merge hooks in
        let mut parsed: serde_json::Value = serde_json::from_str(&existing)
            .map_err(|e| format!("Failed to parse .claude/settings.json: {e}"))?;

        let hook_value: serde_json::Value = serde_json::from_str(CLAUDE_CODE_HOOK_SETTINGS)
            .expect("built-in hook template is valid JSON");

        if let Some(obj) = parsed.as_object_mut()
            && let Some(hooks) = hook_value.get("hooks")
        {
            obj.insert("hooks".to_string(), hooks.clone());
        }

        let new_content = serde_json::to_string_pretty(&parsed)
            .map_err(|e| format!("Failed to serialize settings: {e}"))?;
        fs::write(&path, format!("{new_content}\n"))
            .map_err(|e| format!("Failed to write .claude/settings.json: {e}"))?;
    } else {
        fs::write(&path, format!("{CLAUDE_CODE_HOOK_SETTINGS}\n"))
            .map_err(|e| format!("Failed to create .claude/settings.json: {e}"))?;
    }

    Ok(true)
}

/// Remove a section starting with `marker` from a file.
/// If the file becomes empty, delete it.
fn remove_section_from_file(path: &Path, marker: &str) -> Result<bool, String> {
    if !path.exists() {
        return Ok(false);
    }

    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {e}", path.display()))?;

    if !content.contains(marker) {
        return Ok(false);
    }

    // Find the marker and remove everything from it to end-of-file or next top-level heading
    let mut lines: Vec<&str> = content.lines().collect();
    let mut start = None;
    let mut end = lines.len();

    for (i, line) in lines.iter().enumerate() {
        if line.contains(marker) {
            start = Some(i);
            // Look for the next top-level heading that isn't part of our section
            for (j, line) in lines.iter().enumerate().skip(i + 1) {
                if line.starts_with("# ") && !line.contains("Spec-Sync") {
                    end = j;
                    break;
                }
            }
            break;
        }
    }

    if let Some(start) = start {
        // Remove trailing blank lines before our section too
        let mut actual_start = start;
        while actual_start > 0 && lines[actual_start - 1].trim().is_empty() {
            actual_start -= 1;
        }
        lines.drain(actual_start..end);
    }

    let new_content = lines.join("\n");
    let trimmed = new_content.trim();

    if trimmed.is_empty() {
        fs::remove_file(path).map_err(|e| format!("Failed to remove {}: {e}", path.display()))?;
    } else {
        fs::write(path, format!("{trimmed}\n"))
            .map_err(|e| format!("Failed to write {}: {e}", path.display()))?;
    }

    Ok(true)
}

// ─── CLI command handlers ────────────────────────────────────────────────────

/// Install hooks for the specified targets (or all if empty).
pub fn cmd_install(root: &Path, targets: &[HookTarget]) {
    let targets = if targets.is_empty() {
        HookTarget::all().to_vec()
    } else {
        targets.to_vec()
    };

    println!(
        "\n--- {} ------------------------------------------------",
        "Installing Hooks".bold()
    );

    let mut installed = 0;
    let mut skipped = 0;
    let mut errors = 0;

    for target in &targets {
        match install_hook(root, *target) {
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
        println!("{installed} hook(s) installed.");
    }
    if skipped > 0 {
        println!("{skipped} hook(s) already present.");
    }
    if errors > 0 {
        println!("{errors} hook(s) failed.");
        std::process::exit(1);
    }
}

/// Uninstall hooks for the specified targets (or all if empty).
pub fn cmd_uninstall(root: &Path, targets: &[HookTarget]) {
    let targets = if targets.is_empty() {
        HookTarget::all().to_vec()
    } else {
        targets.to_vec()
    };

    println!(
        "\n--- {} ------------------------------------------------",
        "Uninstalling Hooks".bold()
    );

    let mut removed = 0;

    for target in &targets {
        match uninstall_hook(root, *target) {
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
        println!("{removed} hook(s) removed.");
    } else {
        println!("No hooks to remove.");
    }
}

/// Show status of all hook targets.
pub fn cmd_status(root: &Path) {
    println!(
        "\n--- {} ------------------------------------------------",
        "Hook Status".bold()
    );

    for target in HookTarget::all() {
        let installed = is_installed(root, *target);
        let status = if installed {
            "installed".green().to_string()
        } else {
            "not installed".dimmed().to_string()
        };
        println!("  {:20} {}", target.description(), status);
    }

    println!();
    println!("Install all: specsync hooks install");
    println!("Install one: specsync hooks install --claude --precommit");
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    // ── HookTarget::all ────────────────────────────────────────────

    #[test]
    fn hook_target_all_returns_six_targets() {
        let all = HookTarget::all();
        assert_eq!(all.len(), 6);
    }

    #[test]
    fn hook_target_all_contains_all_variants() {
        let all = HookTarget::all();
        assert!(all.contains(&HookTarget::Claude));
        assert!(all.contains(&HookTarget::Cursor));
        assert!(all.contains(&HookTarget::Copilot));
        assert!(all.contains(&HookTarget::Agents));
        assert!(all.contains(&HookTarget::Precommit));
        assert!(all.contains(&HookTarget::ClaudeCodeHook));
    }

    // ── HookTarget::name ───────────────────────────────────────────

    #[test]
    fn hook_target_name_returns_expected_strings() {
        assert_eq!(HookTarget::Claude.name(), "claude");
        assert_eq!(HookTarget::Cursor.name(), "cursor");
        assert_eq!(HookTarget::Copilot.name(), "copilot");
        assert_eq!(HookTarget::Agents.name(), "agents");
        assert_eq!(HookTarget::Precommit.name(), "precommit");
        assert_eq!(HookTarget::ClaudeCodeHook.name(), "claude-code-hook");
    }

    // ── HookTarget::description ────────────────────────────────────

    #[test]
    fn hook_target_description_returns_human_readable() {
        assert_eq!(
            HookTarget::Claude.description(),
            "CLAUDE.md agent instructions"
        );
        assert_eq!(HookTarget::Precommit.description(), "Git pre-commit hook");
        assert_eq!(
            HookTarget::ClaudeCodeHook.description(),
            "Claude Code settings.json hook"
        );
    }

    // ── HookTarget::from_str ───────────────────────────────────────

    #[test]
    fn from_str_parses_all_targets() {
        assert_eq!(HookTarget::from_str("claude"), Some(HookTarget::Claude));
        assert_eq!(HookTarget::from_str("cursor"), Some(HookTarget::Cursor));
        assert_eq!(HookTarget::from_str("copilot"), Some(HookTarget::Copilot));
        assert_eq!(HookTarget::from_str("agents"), Some(HookTarget::Agents));
        assert_eq!(
            HookTarget::from_str("precommit"),
            Some(HookTarget::Precommit)
        );
        assert_eq!(
            HookTarget::from_str("claude-code-hook"),
            Some(HookTarget::ClaudeCodeHook)
        );
    }

    #[test]
    fn from_str_is_case_insensitive() {
        assert_eq!(HookTarget::from_str("CLAUDE"), Some(HookTarget::Claude));
        assert_eq!(HookTarget::from_str("Cursor"), Some(HookTarget::Cursor));
        assert_eq!(
            HookTarget::from_str("PreCommit"),
            Some(HookTarget::Precommit)
        );
    }

    #[test]
    fn from_str_accepts_aliases() {
        assert_eq!(
            HookTarget::from_str("pre-commit"),
            Some(HookTarget::Precommit)
        );
        assert_eq!(
            HookTarget::from_str("claude-hook"),
            Some(HookTarget::ClaudeCodeHook)
        );
    }

    #[test]
    fn from_str_returns_none_for_unknown() {
        assert_eq!(HookTarget::from_str("unknown"), None);
        assert_eq!(HookTarget::from_str(""), None);
        assert_eq!(HookTarget::from_str("windsurf"), None);
    }

    // ── is_installed ───────────────────────────────────────────────

    #[test]
    fn is_installed_returns_false_for_empty_dir() {
        let tmp = setup();
        for target in HookTarget::all() {
            assert!(
                !is_installed(tmp.path(), *target),
                "expected not installed: {:?}",
                target
            );
        }
    }

    #[test]
    fn is_installed_claude_detects_marker() {
        let tmp = setup();
        let path = tmp.path().join("CLAUDE.md");
        fs::write(&path, "# Spec-Sync Integration\nSome content").unwrap();
        assert!(is_installed(tmp.path(), HookTarget::Claude));
    }

    #[test]
    fn is_installed_claude_false_without_marker() {
        let tmp = setup();
        let path = tmp.path().join("CLAUDE.md");
        fs::write(&path, "# Some other content\nNo spec-sync here").unwrap();
        assert!(!is_installed(tmp.path(), HookTarget::Claude));
    }

    #[test]
    fn is_installed_cursor_detects_marker() {
        let tmp = setup();
        let path = tmp.path().join(".cursorrules");
        fs::write(&path, "# Spec-Sync Rules\nSome content").unwrap();
        assert!(is_installed(tmp.path(), HookTarget::Cursor));
    }

    #[test]
    fn is_installed_copilot_detects_marker() {
        let tmp = setup();
        let github_dir = tmp.path().join(".github");
        fs::create_dir_all(&github_dir).unwrap();
        fs::write(
            github_dir.join("copilot-instructions.md"),
            "# Spec-Sync Integration",
        )
        .unwrap();
        assert!(is_installed(tmp.path(), HookTarget::Copilot));
    }

    #[test]
    fn is_installed_agents_detects_marker() {
        let tmp = setup();
        fs::write(
            tmp.path().join("AGENTS.md"),
            "# Spec-Sync Integration\ncontent",
        )
        .unwrap();
        assert!(is_installed(tmp.path(), HookTarget::Agents));
    }

    #[test]
    fn is_installed_precommit_detects_marker() {
        let tmp = setup();
        let hooks_dir = tmp.path().join(".git").join("hooks");
        fs::create_dir_all(&hooks_dir).unwrap();
        fs::write(
            hooks_dir.join("pre-commit"),
            "#!/bin/sh\n# spec-sync pre-commit hook\nspecsync check",
        )
        .unwrap();
        assert!(is_installed(tmp.path(), HookTarget::Precommit));
    }

    #[test]
    fn is_installed_claude_code_hook_detects_marker() {
        let tmp = setup();
        let claude_dir = tmp.path().join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();
        fs::write(claude_dir.join("settings.json"), r#"{"hooks":{"PostToolUse":[{"matcher":"Edit","hooks":[{"type":"command","command":"specsync check"}]}]}}"#).unwrap();
        assert!(is_installed(tmp.path(), HookTarget::ClaudeCodeHook));
    }

    // ── install_hook ───────────────────────────────────────────────

    #[test]
    fn install_claude_creates_file() {
        let tmp = setup();
        let result = install_hook(tmp.path(), HookTarget::Claude).unwrap();
        assert!(result);
        let content = fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
        assert!(content.contains("Spec-Sync Integration"));
        assert!(content.contains("specsync check"));
    }

    #[test]
    fn install_claude_appends_to_existing() {
        let tmp = setup();
        let path = tmp.path().join("CLAUDE.md");
        fs::write(&path, "# My Project\n\nExisting content here.").unwrap();
        let result = install_hook(tmp.path(), HookTarget::Claude).unwrap();
        assert!(result);
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.starts_with("# My Project"));
        assert!(content.contains("Spec-Sync Integration"));
    }

    #[test]
    fn install_claude_is_idempotent() {
        let tmp = setup();
        assert!(install_hook(tmp.path(), HookTarget::Claude).unwrap());
        assert!(!install_hook(tmp.path(), HookTarget::Claude).unwrap());
    }

    #[test]
    fn install_cursor_creates_file() {
        let tmp = setup();
        assert!(install_hook(tmp.path(), HookTarget::Cursor).unwrap());
        let content = fs::read_to_string(tmp.path().join(".cursorrules")).unwrap();
        assert!(content.contains("Spec-Sync Rules"));
    }

    #[test]
    fn install_copilot_creates_github_dir() {
        let tmp = setup();
        assert!(install_hook(tmp.path(), HookTarget::Copilot).unwrap());
        assert!(
            tmp.path()
                .join(".github")
                .join("copilot-instructions.md")
                .exists()
        );
        let content =
            fs::read_to_string(tmp.path().join(".github").join("copilot-instructions.md")).unwrap();
        assert!(content.contains("Spec-Sync Integration"));
    }

    #[test]
    fn install_agents_creates_file() {
        let tmp = setup();
        assert!(install_hook(tmp.path(), HookTarget::Agents).unwrap());
        let content = fs::read_to_string(tmp.path().join("AGENTS.md")).unwrap();
        assert!(content.contains("Spec-Sync Integration"));
    }

    #[test]
    fn install_precommit_creates_hook_file() {
        let tmp = setup();
        // Need .git/hooks directory structure
        fs::create_dir_all(tmp.path().join(".git").join("hooks")).unwrap();
        assert!(install_hook(tmp.path(), HookTarget::Precommit).unwrap());
        let path = tmp.path().join(".git").join("hooks").join("pre-commit");
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("spec-sync pre-commit hook"));
        assert!(content.contains("specsync check"));
    }

    #[test]
    fn install_precommit_appends_to_existing_hook() {
        let tmp = setup();
        let hooks_dir = tmp.path().join(".git").join("hooks");
        fs::create_dir_all(&hooks_dir).unwrap();
        fs::write(
            hooks_dir.join("pre-commit"),
            "#!/bin/sh\necho 'existing hook'",
        )
        .unwrap();
        assert!(install_hook(tmp.path(), HookTarget::Precommit).unwrap());
        let content = fs::read_to_string(hooks_dir.join("pre-commit")).unwrap();
        assert!(content.contains("existing hook"));
        assert!(content.contains("spec-sync pre-commit hook"));
    }

    #[test]
    fn install_precommit_creates_hooks_dir_if_missing() {
        let tmp = setup();
        // Don't create .git/hooks — let install do it
        assert!(install_hook(tmp.path(), HookTarget::Precommit).unwrap());
        assert!(
            tmp.path()
                .join(".git")
                .join("hooks")
                .join("pre-commit")
                .exists()
        );
    }

    #[cfg(unix)]
    #[test]
    fn install_precommit_sets_executable_permission() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = setup();
        install_hook(tmp.path(), HookTarget::Precommit).unwrap();
        let path = tmp.path().join(".git").join("hooks").join("pre-commit");
        let perms = fs::metadata(&path).unwrap().permissions();
        assert_eq!(perms.mode() & 0o755, 0o755);
    }

    #[test]
    fn install_claude_code_hook_creates_settings() {
        let tmp = setup();
        assert!(install_hook(tmp.path(), HookTarget::ClaudeCodeHook).unwrap());
        let path = tmp.path().join(".claude").join("settings.json");
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("specsync check"));
    }

    #[test]
    fn install_claude_code_hook_merges_into_existing() {
        let tmp = setup();
        let claude_dir = tmp.path().join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();
        fs::write(claude_dir.join("settings.json"), r#"{"existingKey": true}"#).unwrap();
        assert!(install_hook(tmp.path(), HookTarget::ClaudeCodeHook).unwrap());
        let content = fs::read_to_string(claude_dir.join("settings.json")).unwrap();
        assert!(content.contains("existingKey"));
        assert!(content.contains("specsync check"));
    }

    #[test]
    fn install_claude_code_hook_idempotent() {
        let tmp = setup();
        assert!(install_hook(tmp.path(), HookTarget::ClaudeCodeHook).unwrap());
        assert!(!install_hook(tmp.path(), HookTarget::ClaudeCodeHook).unwrap());
    }

    // ── uninstall_hook ─────────────────────────────────────────────

    #[test]
    fn uninstall_returns_false_when_not_installed() {
        let tmp = setup();
        assert!(!uninstall_hook(tmp.path(), HookTarget::Claude).unwrap());
    }

    #[test]
    fn uninstall_claude_removes_section() {
        let tmp = setup();
        install_hook(tmp.path(), HookTarget::Claude).unwrap();
        assert!(is_installed(tmp.path(), HookTarget::Claude));
        let result = uninstall_hook(tmp.path(), HookTarget::Claude).unwrap();
        assert!(result);
        // File should be removed since it only had our content
        assert!(!tmp.path().join("CLAUDE.md").exists());
    }

    #[test]
    fn uninstall_claude_preserves_other_content() {
        let tmp = setup();
        let path = tmp.path().join("CLAUDE.md");
        fs::write(&path, "# My Project\n\nExisting rules.\n").unwrap();
        install_hook(tmp.path(), HookTarget::Claude).unwrap();
        uninstall_hook(tmp.path(), HookTarget::Claude).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("My Project"));
        assert!(!content.contains("Spec-Sync Integration"));
    }

    #[test]
    fn uninstall_cursor_removes_section() {
        let tmp = setup();
        install_hook(tmp.path(), HookTarget::Cursor).unwrap();
        let result = uninstall_hook(tmp.path(), HookTarget::Cursor).unwrap();
        assert!(result);
        assert!(!tmp.path().join(".cursorrules").exists());
    }

    #[test]
    fn uninstall_precommit_removes_hook_file() {
        let tmp = setup();
        install_hook(tmp.path(), HookTarget::Precommit).unwrap();
        let result = uninstall_hook(tmp.path(), HookTarget::Precommit).unwrap();
        assert!(result);
        assert!(
            !tmp.path()
                .join(".git")
                .join("hooks")
                .join("pre-commit")
                .exists()
        );
    }

    #[test]
    fn uninstall_claude_code_hook_is_refused() {
        let tmp = setup();
        install_hook(tmp.path(), HookTarget::ClaudeCodeHook).unwrap();
        let result = uninstall_hook(tmp.path(), HookTarget::ClaudeCodeHook);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("manually"));
    }

    // ── remove_section_from_file ───────────────────────────────────

    #[test]
    fn remove_section_deletes_file_if_empty_after() {
        let tmp = setup();
        let path = tmp.path().join("test.md");
        fs::write(&path, "# Spec-Sync Integration\nSome content\n").unwrap();
        let result = remove_section_from_file(&path, "# Spec-Sync Integration").unwrap();
        assert!(result);
        assert!(!path.exists());
    }

    #[test]
    fn remove_section_preserves_content_before_marker() {
        let tmp = setup();
        let path = tmp.path().join("test.md");
        fs::write(
            &path,
            "# My Project\n\nKeep this.\n\n# Spec-Sync Integration\nRemove this.\n",
        )
        .unwrap();
        remove_section_from_file(&path, "# Spec-Sync Integration").unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("My Project"));
        assert!(content.contains("Keep this"));
        assert!(!content.contains("Spec-Sync Integration"));
    }

    #[test]
    fn remove_section_returns_false_for_missing_marker() {
        let tmp = setup();
        let path = tmp.path().join("test.md");
        fs::write(&path, "# No marker here\n").unwrap();
        assert!(!remove_section_from_file(&path, "# Spec-Sync Integration").unwrap());
    }

    #[test]
    fn remove_section_returns_false_for_missing_file() {
        let tmp = setup();
        let path = tmp.path().join("nonexistent.md");
        assert!(!remove_section_from_file(&path, "# Spec-Sync Integration").unwrap());
    }

    #[test]
    fn remove_section_stops_at_next_top_level_heading() {
        let tmp = setup();
        let path = tmp.path().join("test.md");
        fs::write(
            &path,
            "# Before\n\nKeep.\n\n# Spec-Sync Integration\nRemove.\n\n# After\n\nAlso keep.\n",
        )
        .unwrap();
        remove_section_from_file(&path, "# Spec-Sync Integration").unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("Before"));
        assert!(content.contains("Also keep"));
        assert!(!content.contains("Spec-Sync Integration"));
    }
}
