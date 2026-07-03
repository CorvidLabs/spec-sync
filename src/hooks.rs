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

Run `specsync add-spec <module-name>` to scaffold the spec and companion files, then complete the spec before writing code.

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

Run `specsync add-spec <module-name>` to scaffold the spec and companion files, then complete the spec before writing code.

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

/// The exact hook command installed into `.claude/settings.json`. Used as the
/// install / is-installed marker: a precise match, so merely referencing specsync
/// elsewhere (e.g. `Bash(specsync:*)` in `permissions`) is NOT mistaken for an
/// installed hook. Must stay in sync with the command in CLAUDE_CODE_HOOK_SETTINGS.
const CLAUDE_CODE_HOOK_MARKER: &str = "specsync check --json 2>/dev/null | head -1 || true";

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
                    .map(|c| c.contains(CLAUDE_CODE_HOOK_MARKER))
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
            remove_section_from_file(&path, "# Spec-Sync Integration", CLAUDE_MD_SNIPPET)
        }
        HookTarget::Cursor => {
            let path = root.join(".cursorrules");
            remove_section_from_file(&path, "# Spec-Sync Rules", CURSORRULES_SNIPPET)
        }
        HookTarget::Copilot => {
            let path = root.join(".github").join("copilot-instructions.md");
            remove_section_from_file(
                &path,
                "# Spec-Sync Integration",
                COPILOT_INSTRUCTIONS_SNIPPET,
            )
        }
        HookTarget::Agents => {
            let path = root.join("AGENTS.md");
            remove_section_from_file(&path, "# Spec-Sync Integration", AGENTS_MD_SNIPPET)
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

/// Sentinels bracketing the spec-sync-managed block in a markdown instruction
/// file. They let `uninstall` remove EXACTLY the block, without guessing where it
/// ends — so user content added after the block is never swept away. HTML comments
/// render invisibly in markdown.
const HOOK_BEGIN: &str =
    "<!-- specsync:hook:begin (managed by `specsync hooks` — do not edit inside) -->";
const HOOK_END: &str = "<!-- specsync:hook:end -->";

/// Wrap a managed snippet in begin/end sentinels.
fn wrap_snippet(snippet: &str) -> String {
    format!("{HOOK_BEGIN}\n{}\n{HOOK_END}", snippet.trim())
}

/// Append the sentinel-wrapped snippet to `path`, or create the file with it.
/// No-op (Ok(false)) if a spec-sync block is already present.
fn install_markdown_snippet(path: &Path, snippet: &str, label: &str) -> Result<bool, String> {
    let wrapped = wrap_snippet(snippet);
    if path.exists() {
        let existing =
            fs::read_to_string(path).map_err(|e| format!("Failed to read {label}: {e}"))?;
        if existing.contains("Spec-Sync") {
            return Ok(false);
        }
        let new_content = format!("{}\n\n{wrapped}\n", existing.trim_end());
        fs::write(path, new_content).map_err(|e| format!("Failed to write {label}: {e}"))?;
    } else {
        fs::write(path, format!("{wrapped}\n"))
            .map_err(|e| format!("Failed to create {label}: {e}"))?;
    }
    Ok(true)
}

fn install_claude_md(root: &Path) -> Result<bool, String> {
    install_markdown_snippet(&root.join("CLAUDE.md"), CLAUDE_MD_SNIPPET, "CLAUDE.md")
}

fn install_cursorrules(root: &Path) -> Result<bool, String> {
    install_markdown_snippet(
        &root.join(".cursorrules"),
        CURSORRULES_SNIPPET,
        ".cursorrules",
    )
}

fn install_copilot(root: &Path) -> Result<bool, String> {
    let github_dir = root.join(".github");
    if !github_dir.exists() {
        fs::create_dir_all(&github_dir).map_err(|e| format!("Failed to create .github/: {e}"))?;
    }
    install_markdown_snippet(
        &github_dir.join("copilot-instructions.md"),
        COPILOT_INSTRUCTIONS_SNIPPET,
        "copilot-instructions.md",
    )
}

fn install_agents_md(root: &Path) -> Result<bool, String> {
    install_markdown_snippet(&root.join("AGENTS.md"), AGENTS_MD_SNIPPET, "AGENTS.md")
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

        // Precise marker (the full hook command), not a bare "specsync" substring —
        // otherwise a user who merely allows `Bash(specsync:*)` in `permissions`, or
        // references specsync anywhere else, is wrongly treated as already-installed
        // and never gets the hook.
        if existing.contains(CLAUDE_CODE_HOOK_MARKER) {
            return Ok(false);
        }

        // Parse existing JSON, merge hooks in
        let mut parsed: serde_json::Value = serde_json::from_str(&existing)
            .map_err(|e| format!("Failed to parse .claude/settings.json: {e}"))?;

        let hook_value: serde_json::Value = serde_json::from_str(CLAUDE_CODE_HOOK_SETTINGS)
            .expect("built-in hook template is valid JSON");

        if let Some(obj) = parsed.as_object_mut()
            && let Some(new_hooks) = hook_value.get("hooks").and_then(|h| h.as_object())
        {
            merge_claude_code_hooks(obj, new_hooks);
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

/// Deep-merge specsync's hook events into an existing `.claude/settings.json`
/// object WITHOUT clobbering the user's other settings or hooks.
///
/// Claude Code's `hooks` maps an event name (e.g. `PostToolUse`) to an array of
/// matcher groups. We merge per-event: an event the user doesn't have is added;
/// an event they do have has our matcher group(s) APPENDED to their array, so
/// their existing hooks — and any other events like `PreToolUse` — survive.
fn merge_claude_code_hooks(
    settings: &mut serde_json::Map<String, serde_json::Value>,
    new_hooks: &serde_json::Map<String, serde_json::Value>,
) {
    let existing = settings
        .entry("hooks".to_string())
        .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));

    // If the user's `hooks` is present but not an object (malformed for Claude
    // Code), replace it with a working object rather than merge into a non-map.
    if !existing.is_object() {
        *existing = serde_json::Value::Object(serde_json::Map::new());
    }
    let Some(existing_obj) = existing.as_object_mut() else {
        return;
    };

    for (event, groups) in new_hooks {
        match existing_obj.get_mut(event) {
            Some(serde_json::Value::Array(existing_arr)) => {
                if let Some(new_arr) = groups.as_array() {
                    existing_arr.extend(new_arr.iter().cloned());
                }
            }
            // Event absent, or present but not an array → set our value.
            _ => {
                existing_obj.insert(event.clone(), groups.clone());
            }
        }
    }
}

/// Index of the first contiguous run in `haystack` matching `needle`, comparing
/// lines by trailing-trimmed content. `None` if absent or `needle` is empty/longer.
fn position_of_slice(haystack: &[&str], needle: &[&str]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| {
        w.iter()
            .zip(needle)
            .all(|(a, b)| a.trim_end() == b.trim_end())
    })
}

/// Remove the spec-sync-managed section from a file, preserving everything else.
///
/// Three strategies, most precise first:
/// 1. **Sentinels** — blocks we wrote carry `begin`/`end` comments; remove exactly
///    between them. User content before OR after the block is untouched.
/// 2. **Exact snippet** — a legacy block (no sentinels) installed from a `snippet`
///    we still recognize is removed by matching that exact text, so real legacy
///    installs are just as safe as new ones.
/// 3. **Heuristic** — only when the snippet has drifted beyond recognition: remove
///    from the marker to the next LEVEL-1 heading that isn't ours, or end-of-file.
///    Never deletes a file that had content before the block.
fn remove_section_from_file(path: &Path, marker: &str, snippet: &str) -> Result<bool, String> {
    if !path.exists() {
        return Ok(false);
    }

    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {e}", path.display()))?;

    if !content.contains(marker) {
        return Ok(false);
    }

    // Preserve the file's dominant line ending rather than rewriting CRLF→LF.
    let newline = if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let mut lines: Vec<&str> = content.lines().collect();

    let begin = lines.iter().position(|l| l.contains(HOOK_BEGIN));
    let end_marker = lines.iter().rposition(|l| l.contains(HOOK_END));
    let snippet_lines: Vec<&str> = snippet.trim().lines().collect();

    let (start, end, precise) = if let (Some(b), Some(e)) = (begin, end_marker)
        && e >= b
    {
        // 1. Precise removal between explicit sentinels.
        (b, e + 1, true)
    } else if let Some(s) = position_of_slice(&lines, &snippet_lines) {
        // 2. Exact known snippet (legacy install we recognize) — precise.
        (s, s + snippet_lines.len(), true)
    } else {
        // 3. Heuristic fallback: stop at the next LEVEL-1 heading that isn't ours
        //    (the block's own sub-sections are level 2+), or end-of-file.
        let start = lines.iter().position(|l| l.contains(marker)).unwrap_or(0);
        let mut end = lines.len();
        for (j, line) in lines.iter().enumerate().skip(start + 1) {
            if line.starts_with("# ") && !line.contains("Spec-Sync") {
                end = j;
                break;
            }
        }
        (start, end, false)
    };

    // Also consume blank lines immediately before the block so no gap is left.
    let mut actual_start = start;
    while actual_start > 0 && lines[actual_start - 1].trim().is_empty() {
        actual_start -= 1;
    }
    lines.drain(actual_start..end);

    let new_content = lines.join(newline);
    let trimmed = new_content.trim();

    if trimmed.is_empty() {
        // Delete the file only when it's safe to conclude it held nothing but our
        // block: a precise removal (sentinel or exact snippet), or a legacy block
        // that began at the very top. If any content preceded the block, keep it.
        if precise || actual_start == 0 {
            fs::remove_file(path)
                .map_err(|e| format!("Failed to remove {}: {e}", path.display()))?;
        } else {
            fs::write(path, "").map_err(|e| format!("Failed to write {}: {e}", path.display()))?;
        }
    } else {
        fs::write(path, format!("{trimmed}{newline}"))
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
        // The fixture must contain the ACTUAL installed hook command; is_installed
        // matches the precise marker so an unrelated `specsync check` reference is
        // not mistaken for an installed hook.
        let tmp = setup();
        let claude_dir = tmp.path().join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();
        fs::write(claude_dir.join("settings.json"), CLAUDE_CODE_HOOK_SETTINGS).unwrap();
        assert!(is_installed(tmp.path(), HookTarget::ClaudeCodeHook));

        // A settings.json that references specsync only loosely is NOT "installed".
        fs::write(
            claude_dir.join("settings.json"),
            r#"{"hooks":{"PostToolUse":[{"matcher":"Edit","hooks":[{"type":"command","command":"specsync check"}]}]}}"#,
        )
        .unwrap();
        assert!(!is_installed(tmp.path(), HookTarget::ClaudeCodeHook));
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
    fn install_claude_code_hook_preserves_existing_user_hooks() {
        // Regression (H3): install must DEEP-MERGE, never clobber the user's own
        // `hooks` object. A pre-existing PreToolUse event and a user's own
        // PostToolUse matcher must both survive, with specsync's group appended.
        let tmp = setup();
        let claude_dir = tmp.path().join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();
        let user = r#"{
  "model": "opus",
  "hooks": {
    "PreToolUse": [
      { "matcher": "Bash", "hooks": [{ "type": "command", "command": "my-guard.sh" }] }
    ],
    "PostToolUse": [
      { "matcher": "Read", "hooks": [{ "type": "command", "command": "my-logger.sh" }] }
    ]
  }
}"#;
        fs::write(claude_dir.join("settings.json"), user).unwrap();

        assert!(install_hook(tmp.path(), HookTarget::ClaudeCodeHook).unwrap());

        let parsed: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(claude_dir.join("settings.json")).unwrap())
                .unwrap();
        // Unrelated top-level setting preserved.
        assert_eq!(parsed["model"], "opus");
        // The user's PreToolUse event is untouched.
        let pre = parsed["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(pre.len(), 1);
        assert_eq!(pre[0]["hooks"][0]["command"], "my-guard.sh");
        // The user's own PostToolUse group survives AND specsync's is appended.
        let post = parsed["hooks"]["PostToolUse"].as_array().unwrap();
        assert_eq!(post.len(), 2, "user group + specsync group");
        assert_eq!(post[0]["hooks"][0]["command"], "my-logger.sh");
        assert!(
            post[1]["hooks"][0]["command"]
                .as_str()
                .unwrap()
                .contains("specsync check"),
            "specsync group appended, not replacing the user's"
        );
    }

    #[test]
    fn install_claude_code_hook_not_skipped_by_specsync_permission() {
        // The install guard must match the actual hook command, not a bare
        // "specsync" substring — a user who merely allows `Bash(specsync:*)` must
        // still get the hook installed (previously this was a silent no-op).
        let tmp = setup();
        let claude_dir = tmp.path().join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();
        fs::write(
            claude_dir.join("settings.json"),
            r#"{ "permissions": { "allow": ["Bash(specsync:*)"] } }"#,
        )
        .unwrap();

        assert!(
            install_hook(tmp.path(), HookTarget::ClaudeCodeHook).unwrap(),
            "install must proceed, not skip as already-installed"
        );
        let parsed: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(claude_dir.join("settings.json")).unwrap())
                .unwrap();
        assert_eq!(parsed["permissions"]["allow"][0], "Bash(specsync:*)");
        assert!(
            parsed["hooks"]["PostToolUse"][0]["hooks"][0]["command"]
                .as_str()
                .unwrap()
                .contains("specsync check"),
            "the hook must actually be installed"
        );
        // And now it IS idempotent (marker present).
        assert!(!install_hook(tmp.path(), HookTarget::ClaudeCodeHook).unwrap());
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
        let result =
            remove_section_from_file(&path, "# Spec-Sync Integration", CLAUDE_MD_SNIPPET).unwrap();
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
        remove_section_from_file(&path, "# Spec-Sync Integration", CLAUDE_MD_SNIPPET).unwrap();
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
        assert!(
            !remove_section_from_file(&path, "# Spec-Sync Integration", CLAUDE_MD_SNIPPET).unwrap()
        );
    }

    #[test]
    fn remove_section_returns_false_for_missing_file() {
        let tmp = setup();
        let path = tmp.path().join("nonexistent.md");
        assert!(
            !remove_section_from_file(&path, "# Spec-Sync Integration", CLAUDE_MD_SNIPPET).unwrap()
        );
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
        remove_section_from_file(&path, "# Spec-Sync Integration", CLAUDE_MD_SNIPPET).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("Before"));
        assert!(content.contains("Also keep"));
        assert!(!content.contains("Spec-Sync Integration"));
    }

    #[test]
    fn uninstall_preserves_user_content_after_wrapped_block() {
        // The finding's repro: a `## Deploy Notes` (level-2) block added after the
        // managed block used to be swept to EOF. With sentinels it must survive.
        let tmp = setup();
        let path = tmp.path().join("CLAUDE.md");
        let block = wrap_snippet("# Spec-Sync Integration\nmanaged body\n\n## Commands\nrun");
        fs::write(&path, format!("{block}\n\n## Deploy Notes\nkeep me\n")).unwrap();

        assert!(
            remove_section_from_file(&path, "# Spec-Sync Integration", CLAUDE_MD_SNIPPET).unwrap()
        );
        let content = fs::read_to_string(&path).unwrap();
        assert!(
            content.contains("Deploy Notes") && content.contains("keep me"),
            "user content after the block must survive: {content:?}"
        );
        assert!(!content.contains("managed body"));
        assert!(!content.contains(HOOK_BEGIN) && !content.contains(HOOK_END));
    }

    #[test]
    fn uninstall_preserves_content_before_and_after_wrapped_block() {
        let tmp = setup();
        let path = tmp.path().join("AGENTS.md");
        let block = wrap_snippet("# Spec-Sync Integration\nbody");
        fs::write(&path, format!("# Mine\n\nbefore\n\n{block}\n\nafter\n")).unwrap();

        remove_section_from_file(&path, "# Spec-Sync Integration", CLAUDE_MD_SNIPPET).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert!(
            content.contains("Mine") && content.contains("before") && content.contains("after")
        );
        assert!(!content.contains("Spec-Sync Integration"));
    }

    #[test]
    fn uninstall_wrapped_block_only_deletes_file() {
        let tmp = setup();
        let path = tmp.path().join("CLAUDE.md");
        fs::write(
            &path,
            format!("{}\n", wrap_snippet("# Spec-Sync Integration\nbody")),
        )
        .unwrap();

        remove_section_from_file(&path, "# Spec-Sync Integration", CLAUDE_MD_SNIPPET).unwrap();
        assert!(
            !path.exists(),
            "a file holding only our block should be removed"
        );
    }

    #[test]
    fn uninstall_legacy_block_without_sentinels_preserves_appended_notes() {
        // A pre-sentinel install: the raw snippet written verbatim (no begin/end
        // comments), then the user appended a level-2 section. Matching the exact
        // known snippet must remove only the block and keep the notes and file.
        let tmp = setup();
        let path = tmp.path().join("CLAUDE.md");
        fs::write(
            &path,
            format!("{}\n\n## Deploy Notes\nkeep me\n", CLAUDE_MD_SNIPPET.trim()),
        )
        .unwrap();

        assert!(
            remove_section_from_file(&path, "# Spec-Sync Integration", CLAUDE_MD_SNIPPET).unwrap()
        );
        assert!(
            path.exists(),
            "legacy block-plus-notes must not delete the file"
        );
        let content = fs::read_to_string(&path).unwrap();
        assert!(
            content.contains("Deploy Notes") && content.contains("keep me"),
            "notes after a legacy block must survive: {content:?}"
        );
        assert!(!content.contains("Spec-Sync"), "block removed: {content:?}");
    }

    #[test]
    fn uninstall_preserves_crlf_line_endings() {
        let tmp = setup();
        let path = tmp.path().join("CLAUDE.md");
        let block = wrap_snippet("# Spec-Sync Integration\nbody").replace('\n', "\r\n");
        fs::write(
            &path,
            format!("# Mine\r\n\r\nbefore\r\n\r\n{block}\r\n\r\nafter\r\n"),
        )
        .unwrap();

        remove_section_from_file(&path, "# Spec-Sync Integration", CLAUDE_MD_SNIPPET).unwrap();
        let after = fs::read_to_string(&path).unwrap();
        assert!(after.contains("# Mine") && after.contains("before") && after.contains("after"));
        assert!(!after.contains("Spec-Sync"));
        assert!(
            after.contains("\r\n"),
            "CRLF endings must be preserved: {after:?}"
        );
        assert!(
            !after.replace("\r\n", "").contains('\n'),
            "no bare LF should be introduced: {after:?}"
        );
    }

    #[test]
    fn install_then_uninstall_roundtrip_preserves_appended_notes() {
        // End-to-end through the real install/uninstall for a target.
        let tmp = setup();
        let root = tmp.path();
        assert!(install_hook(root, HookTarget::Claude).unwrap());
        let path = root.join("CLAUDE.md");
        let with_notes = format!(
            "{}\n\n## My Deploy Notes\ndo not delete\n",
            fs::read_to_string(&path).unwrap().trim_end()
        );
        fs::write(&path, with_notes).unwrap();

        assert!(uninstall_hook(root, HookTarget::Claude).unwrap());
        let content = fs::read_to_string(&path).unwrap();
        assert!(
            content.contains("My Deploy Notes") && content.contains("do not delete"),
            "notes appended after install must survive uninstall: {content:?}"
        );
        assert!(!content.contains("Spec-Sync"));
    }
}
