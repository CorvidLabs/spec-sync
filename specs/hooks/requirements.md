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

### REQ-hooks-001

The hooks module SHALL install, inspect, and conservatively remove managed integration content without destroying user-owned content.

Acceptance Criteria
- Six targets supported: Claude, Cursor, Copilot, Agents, Precommit, ClaudeCodeHook
- Installation is idempotent — re-installing an already-installed hook is a no-op returning Ok(false)
- Agent instructions are appended to existing files, not overwritten
- Marker strings ("Spec-Sync Integration", "Spec-Sync Rules") are used to detect existing installations
- Pre-commit hook is made executable (0o755) on Unix systems
- Uninstalling Claude Code hook settings is refused (too risky to modify IDE settings)
- Empty targets list means "all targets"
- Pre-commit hook appends to existing hooks (preserves existing shebang and content)

### REQ-hooks-002

Hook installation SHALL resolve the repository's effective hook directory and manage only
project-keyed SpecSync blocks.

Acceptance Criteria

- Installation honors normal repositories, worktrees, submodules, and `core.hooksPath`.
- Managed blocks carry a stable project key so multiple projects sharing a hook path do not collide.
- Install is idempotent and generated pre-commit checks are strict and blocking.
- Uninstall removes only the matching SpecSync block and preserves user and other-project content.
- Symlink escapes, ambiguous hook roots, and unsafe paths fail before mutation.

### REQ-hooks-two-verb-001

Installed hooks instruction snippets SHALL document `change check` as scoped verification and `change audit` as active-workspace project health, and SHALL NOT instruct agents to treat check as full archive terminal-evidence validation.

Acceptance Criteria
- Claude/Agents.md-style snippets mention both verbs.
- Cursor/Copilot snippets mention the two-verb distinction.

