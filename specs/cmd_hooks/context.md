---
spec: cmd_hooks.spec.md
---

## Key Decisions

- The command is a pure dispatcher. Unlike most commands it does **not** load config or discover specs — it only translates CLI flags into `hooks::HookTarget` values and forwards to the `hooks` library module.
- "No flags means all targets" is encoded by returning an empty `Vec<HookTarget>` from `collect_hook_targets`; the convention is interpreted downstream in the `hooks` module, not here.
- Install and uninstall share the exact same flag-collection helper, guaranteeing symmetric target selection.

## Files to Read First

- `src/commands/hooks.rs` — the dispatcher and `collect_hook_targets`.
- `src/hooks.rs` — `cmd_install`, `cmd_uninstall`, `cmd_status`, the `HookTarget` enum, and all file/IO logic.
- `src/cli.rs` (`HooksAction`) — the flag definitions (`--claude`, `--cursor`, `--copilot`, `--agents`, `--precommit`, `--claude-code-hook`).

## Current Status

Implemented and stable. No unit tests live in this file; the dispatcher is driven end to end by `hooks_uninstall_preserves_user_content_after_block` and `hooks_install_claude_code_hook_preserves_user_settings` in `tests/integration/commands.rs`, and the file/IO behavior is validated through the `hooks` module.

## Notes

- Targets map to: `Claude` (CLAUDE.md), `Cursor` (.cursorrules), `Copilot` (.github/copilot-instructions.md), `Agents` (AGENTS.md), `Precommit` (git pre-commit hook), `ClaudeCodeHook` (Claude Code settings.json hook).
- Part of the command layer — orchestrates a library module rather than containing domain logic.
