---
spec: hooks.spec.md
---

## Key Decisions

- **Idempotent installation**: Installing an already-present hook returns `Ok(false)` rather than erroring or duplicating content. Marker strings ("Spec-Sync Integration", "Spec-Sync Rules") are used for detection.
- **Append, not overwrite**: Agent instruction files (CLAUDE.md, .cursorrules, copilot-instructions.md) are appended to, preserving any existing user content.
- **Pre-commit hook safety**: The hook is appended to existing `.git/hooks/pre-commit` files (skipping the shebang line), and made executable (0o755) on Unix. This respects other hooks in the pipeline.
- **Claude Code hook uninstall refused**: The `settings.json` hook integration is too risky to auto-edit (complex JSON with other user settings), so uninstall prints a manual removal instruction instead.
- **All targets by default**: When no specific `--claude`/`--cursor`/etc. flags are given, an empty targets vec signals "install/uninstall all targets."

## Files to Read First

- `src/hooks.rs` — Single-file module with all hook targets, installation/uninstallation logic, and CLI handlers.

## Current Status

Fully implemented. All 5 hook targets work: CLAUDE.md, .cursorrules, .github/copilot-instructions.md, .git/hooks/pre-commit, and .claude/settings.json.

## Notes

- The hook content includes spec-sync CLI commands and instructions for how AI agents should interact with specs.
- `cmd_status()` reports installed/not-installed for each target, useful for CI verification.
