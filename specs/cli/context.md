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

## Files to Read First

- `src/main.rs` — CLI dispatcher: arg parsing, subcommand routing, output formatting.

## Current Status

Fully implemented. All 12 subcommands work: check, coverage, generate, init, score, watch, mcp, add-spec, init-registry, resolve, hooks (install/uninstall/status).

## Notes

- Every library module is consumed by the CLI — it's the integration point for the entire tool.
- `watch` and `mcp` are the only modules that take over the process (long-running server / file watcher).
