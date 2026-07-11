---
spec: hooks.spec.md
---

## User Stories

- As a developer using Claude Code, I want `specsync hooks install claude` to add spec-sync instructions to CLAUDE.md so that Claude automatically respects my specs when editing code
- As a developer using Cursor, I want spec-sync rules injected into .cursorrules so that Cursor follows spec conventions during code generation
- As a developer using Copilot, I want instructions added to .github/copilot-instructions.md so that Copilot suggestions align with documented APIs
- As a team lead, I want a pre-commit hook that runs `specsync check --strict` so that spec violations are caught before code reaches the remote
- As a developer, I want `specsync hooks install` with no targets to install all hooks at once so that setup is a single command
- As a developer, I want `specsync hooks status` to show which hooks are installed so that I can verify my setup
- As a developer, I want `specsync hooks uninstall` to cleanly remove hooks so that I can disable integration without manual file editing
- As a multi-agent team, I want AGENTS.md instructions installed so that any agent framework can discover spec-sync conventions

## Acceptance Criteria

- Six targets supported: Claude, Cursor, Copilot, Agents, Precommit, ClaudeCodeHook
- Installation is idempotent — re-installing an already-installed hook is a no-op returning Ok(false)
- Agent instructions are appended to existing files, not overwritten
- Marker strings ("Spec-Sync Integration", "Spec-Sync Rules") are used to detect existing installations
- Pre-commit hook is made executable (0o755) on Unix systems
- Uninstalling Claude Code hook settings is refused (too risky to modify IDE settings)
- Empty targets list means "all targets"
- Pre-commit hook appends to existing hooks (preserves existing shebang and content)

## Constraints

- Must not overwrite user content in instruction files — only append spec-sync sections
- Pre-commit hook must be compatible with other hooks in the same file
- File permissions must be set correctly on Unix (executable bit for pre-commit)

## Out of Scope

- Installing hooks for AI tools not in the supported list
- Managing git hook frameworks (husky, lefthook, etc.)
- Modifying IDE settings beyond the Claude Code hook
- Auto-updating hook content when spec-sync is upgraded
