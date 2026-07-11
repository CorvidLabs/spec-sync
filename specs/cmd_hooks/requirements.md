---
spec: cmd_hooks.spec.md
---

## User Stories

- As a developer, I want to install spec-sync agent instructions and a git pre-commit hook with one command so that drift is caught before commit and AI agents know the spec conventions.
- As a developer who only uses one agent, I want to select specific targets (e.g. `--claude --precommit`) so that I don't install files for tools I don't use.
- As a developer, I want a `status` subcommand so that I can see which hooks/instructions are currently installed.
- As a developer, I want to cleanly uninstall what I installed so that I can remove spec-sync's footprint.

## Acceptance Criteria

- `cmd_hooks` dispatches `Install`, `Uninstall`, and `Status` actions to `hooks::cmd_install`, `hooks::cmd_uninstall`, and `hooks::cmd_status` respectively.
- Boolean flags `claude`, `cursor`, `copilot`, `agents`, `precommit`, `claude_code_hook` map one-to-one to `hooks::HookTarget` variants in `collect_hook_targets`.
- When no target flags are set, the collected target vec is empty, which the `hooks` module interprets as "all targets".
- The same flag-to-target mapping is used for both install and uninstall.

## Constraints

- This module is a thin dispatcher: it performs no I/O itself and contains no domain logic — all file writes and status reporting live in the `hooks` library module.
- Must not panic; actual error handling (write failures) is delegated to the `hooks` module.

## Out of Scope

- The content of generated instruction files and the pre-commit hook script (owned by `hooks`).
- Validating that the selected agent tooling is actually installed on the machine.
- Interactive prompts, GUI, or web output.
