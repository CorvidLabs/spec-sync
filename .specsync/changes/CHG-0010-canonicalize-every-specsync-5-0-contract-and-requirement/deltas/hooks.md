## ADDED

### REQUIREMENT REQ-hooks-001

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
