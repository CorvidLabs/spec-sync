---
spec: cli.spec.md
---

## Key Decisions

- **Thin dispatcher**: `main.rs` parses CLI args with `clap`, then routes to `cmd_*` handler functions that orchestrate calls to the library modules. No domain logic lives here — purely argument parsing, output formatting, and exit code management.
- **Default subcommand**: `check` runs when no subcommand is given, making the most common operation the easiest to invoke.
- **JSON mode**: `--json` is a global flag so all commands can produce machine-readable output for CI/scripting.
- **Strict mode**: `--strict` converts warnings to errors, useful for CI pipelines that want zero-warning enforcement.
- **Idempotent init**: Both `init` and `init-registry` check for existing files before writing, preventing accidental overwrites.
- **No network by default**: `resolve` only performs network calls with `--remote`, keeping default behavior offline and fast.
- **Hook targets**: When no specific `--claude`/`--cursor`/etc. flags are given, an empty targets vec signals "all targets" to the hooks module.
- **Panic guard**: `main()` wraps `run()` in `std::panic::catch_unwind` and prints a "please report it" message with the issue tracker URL instead of a raw backtrace.
- **`generate --model`**: a `--model` flag overrides `SPECSYNC_AI_MODEL` and the `aiModel` config field, letting users pin a specific model per invocation; provider resolution itself routes through the reworked `ai` module.

## Files to Read First

- `src/cli.rs` — the clap derive structs (`Cli`, `Command`, `LifecycleAction`, `HooksAction`) and their flags/help text; also holds the parser unit tests.
- `src/main.rs` — `run()` dispatcher: builds `root`, resolves `--json`→format, defaults to `Check`, and matches every `Command` variant to a `commands::*` handler.

## Current Status

Fully implemented. The CLI exposes the `check` default plus init, coverage, generate, score, watch, mcp, add-spec, scaffold, init-registry, resolve, diff, hooks, compact, archive-tasks, view, merge, issues, new, wizard, deps, import, stale, report, comment, rules, changelog, rehash, migrate, and lifecycle (with promote/demote/set/status/history/guard/auto-promote/enforce sub-actions). `generate` now also takes `--model`.

## Notes

- Every library module is consumed by the CLI — it's the integration point for the entire tool.
- `watch` and `mcp` are the only commands that take over the process (long-running file watcher / stdio server).
- The clap grammar lives in `src/cli.rs`; `main.rs` is dispatch-only. Parser-level tests live in `src/cli.rs`; exit-code tests live in `src/main.rs`.
