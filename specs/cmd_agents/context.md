---
spec: cmd_agents.spec.md
---

## Key Decisions

- The command is a pure dispatcher, mirroring `cmd_hooks`. It performs no I/O itself — it only translates CLI flags into `agents::AgentTool` values and forwards to the `agents` library module.
- "No flags means all tools" is encoded by returning an empty `Vec<AgentTool>` from `collect_agent_targets`; the convention is interpreted downstream in the `agents` module, matching `cmd_hooks`'s existing pattern.
- Install and uninstall share the exact same flag-collection helper, guaranteeing symmetric target selection.

## Files to Read First

- `src/commands/agents.rs` — the dispatcher and `collect_agent_targets`.
- `src/agents.rs` — `cmd_install`, `cmd_uninstall`, `cmd_status`, the `AgentTool` enum, and all file/IO logic.
- `src/cli.rs` (`AgentsAction`) — the flag definitions (`--claude`, `--cursor`, `--codex`, `--gemini`).

## Current Status

Implemented and stable. No tests target this file directly; behavior is validated through the `agents` module.

## Notes

- Targets map to: `Claude` (skill + `/specsync:create-spec`), `Cursor` (skill + `/specsync-create-spec`), `Codex` (skill only), `Gemini` (skill + `/specsync:create-spec` TOML command).
- Part of the command layer — orchestrates a library module rather than containing domain logic, same shape as `cmd_hooks`.
