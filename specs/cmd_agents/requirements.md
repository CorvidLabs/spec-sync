---
spec: cmd_agents.spec.md
---

## User Stories

- As a developer, I want to install spec-sync's native agent skills and slash commands with one command so that Claude/Cursor/Codex/Gemini all get set up without one command per tool.
- As a developer who only uses one agent, I want to select specific targets (e.g. `--claude --gemini`) so that I don't install files for tools I don't use.
- As a developer, I want a `status` subcommand so that I can see which tool integrations are currently installed.
- As a developer, I want to cleanly uninstall what I installed so that I can remove spec-sync's footprint from a tool.

## Acceptance Criteria

- `cmd_agents` dispatches `Install`, `Uninstall`, and `Status` actions to `agents::cmd_install`, `agents::cmd_uninstall`, and `agents::cmd_status` respectively.
- Boolean flags `claude`, `cursor`, `codex`, `gemini` map one-to-one to `agents::AgentTool` variants in `collect_agent_targets`.
- When no target flags are set, the collected target vec is empty, which the `agents` module interprets as "all tools".
- The same flag-to-target mapping is used for both install and uninstall.

## Constraints

- This module is a thin dispatcher: it performs no I/O itself and contains no domain logic — all file writes and status reporting live in the `agents` library module.
- Must not panic; actual error handling (write failures) is delegated to the `agents` module.

## Out of Scope

- The content of generated skill/command files (owned by `agents`).
- Validating that the selected agent tooling is actually installed on the machine.
- Interactive prompts, GUI, or web output.
