use colored::Colorize;
use std::fs;
use std::path::Path;

// ─── Agent instruction templates ─────────────────────────────────────────────

const CLAUDE_MD_SNIPPET: &str = r#"# Spec-Sync Integration

This project uses [spec-sync](https://github.com/CorvidLabs/spec-sync) for bidirectional spec-to-code validation.

## Before modifying any module

1. Read the relevant spec in `specs/<module>/<module>.spec.md`
2. Check companion files: `specs/<module>/tasks.md` and `specs/<module>/context.md`
3. After changes, run `specsync check` to verify specs still pass

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

## Rules

- Before editing a module, read its spec at `specs/<module>/<module>.spec.md`
- Check `specs/<module>/tasks.md` for outstanding work and `specs/<module>/context.md` for decisions
- After modifying code, ensure `specsync check` still passes
- When creating new modules, run `specsync add-spec <module-name>` first
- Keep specs in sync: if you change exports, parameters, or types, update the spec's Public API table
- Run `specsync check --strict` before committing
"#;

const COPILOT_INSTRUCTIONS_SNIPPET: &str = r#"# Spec-Sync Integration

This project uses spec-sync for bidirectional spec-to-code validation.

## Guidelines

- Specs are in `specs/<module>/<module>.spec.md` — read the relevant spec before modifying a module
- Companion files `tasks.md` and `context.md` in each spec directory provide additional context
- After changes, `specsync check` should pass with no errors
- New modules need specs: run `specsync add-spec <module-name>`
- Keep the Public API table in each spec up to date with actual exports
"#;

const PRE_COMMIT_HOOK: &str = r#"#!/bin/sh
# spec-sync pre-commit hook — validates specs before allowing commits.
# Installed by: specsync hooks install --precommit
# Remove by deleting this file or running: specsync hooks uninstall --precommit

if command -v specsync >/dev/null 2>&1; then
    echo "specsync: checking specs..."
    if ! specsync check --strict; then
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
    Precommit,
    ClaudeCodeHook,
}

impl HookTarget {
    pub fn all() -> &'static [HookTarget] {
        &[
            HookTarget::Claude,
            HookTarget::Cursor,
            HookTarget::Copilot,
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
            HookTarget::Precommit => "precommit",
            HookTarget::ClaudeCodeHook => "claude-code-hook",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            HookTarget::Claude => "CLAUDE.md agent instructions",
            HookTarget::Cursor => ".cursorrules agent instructions",
            HookTarget::Copilot => ".github/copilot-instructions.md",
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
        HookTarget::Precommit => {
            let path = root.join(".git").join("hooks").join("pre-commit");
            if path.exists() {
                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read pre-commit hook: {e}"))?;
                if content.contains("spec-sync pre-commit hook") {
                    // If the entire file is our hook, remove it
                    if content.trim().starts_with("#!/bin/sh")
                        && content.contains("specsync check")
                        && content.lines().count() < 20
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
