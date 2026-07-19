---
spec: cli_args.spec.md
---

## Key Decisions

- `src/cli.rs` is declaration-only; dispatch lives in `main.rs`.
- Global validation/enforcement flags work before or after subcommands.
- Deterministic `generate` has no provider/model surface; legacy flags fail through Clap.
- Agents, MCP, Lifecycle, and verified Change commands remain first-class.
- Accepted-change reopen is explicit and auditable: the grammar requires both `--actor` and `--reason`.
- Accepted metadata correction is explicit and auditable: `change correct` restricts fields to `public_contract` or `architecture_risk`, values to `yes` or `no`, and requires both `--actor` and `--reason`.
- Acceptance-owner correction is explicit and auditable: `change correct-owner` requires actor and reason plus a batch selection from repeated `--path`/`--spec`, `--manifest`, or `--all-missing`.

## Files to Read First

- `src/cli.rs`
- `src/main.rs`
- `src/commands/generate.rs`

## Current Status

Stable deterministic grammar for the core and agent-native integrations. Help text names the canonical `.specsync/config.toml` layout and all required `new --full` companions; accepted evidence can be reopened, supported accepted classification metadata corrected, or an exact acceptance owner repaired only with explicit audit inputs.
