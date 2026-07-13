---
spec: cli_args.spec.md
---

## Key Decisions

- `src/cli.rs` is declaration-only; dispatch lives in `main.rs`.
- Global validation/enforcement flags work before or after subcommands.
- Deterministic `generate` has no provider/model surface; legacy flags fail through Clap.
- Agents, MCP, Lifecycle, and verified Change commands remain first-class.
- Accepted-change reopen is explicit and auditable: the grammar requires both `--actor` and `--reason`.

## Files to Read First

- `src/cli.rs`
- `src/main.rs`
- `src/commands/generate.rs`

## Current Status

Stable 5.0 grammar for deterministic core and agent-native integrations. Help text names the canonical `.specsync/config.toml` layout and all required `new --full` companions, and accepted evidence can be reopened only with explicit audit inputs.
