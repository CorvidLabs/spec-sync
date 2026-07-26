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
- **Deterministic generation**: `generate` accepts module selection only; coding-agent enrichment is reached through Agents or MCP, not embedded inference flags.
- **Recursive lifecycle boundary**: Before dispatching `change` or `lifecycle`, `main.rs` consults the inherited verification context and exits once with a contextual diagnostic; the default/check path uses the same domain guard through unified checking.
- **MCP capability boundary**: The dispatcher forwards the parsed `allow_write` bit unchanged to the
  MCP server and reports server-root initialization failures on stderr with exit status 2.
- **Retained-root boundary**: MCP and the check/coverage/generate/score/report/comment gates receive
  the validated requested root spelling. Their capability engines bind it before canonicalization,
  so replacing a public symlink/junction alias cannot disappear behind an eagerly canonicalized
  dispatcher path. Generate carries that retained identity through publication so a redirect after
  checked coverage cannot redirect output into a replacement tree.

## Files to Read First

- `src/cli.rs` — the clap derive structs (`Cli`, `Command`, `LifecycleAction`, `HooksAction`) and their flags/help text; also holds the parser unit tests.
- `src/main.rs` — `run()` dispatcher: builds `root`, resolves `--json`→format, defaults to `Check`, blocks recursive lifecycle-family dispatch, and matches every `Command` variant to a `commands::*` handler.

## Current Status

Fully implemented. The CLI exposes deterministic validation/generation, complete lifecycle/change
commands, and native Agents/MCP integration without embedded inference configuration. MCP mutation
remains opt-in at dispatch, startup fails closed before request processing when the root is invalid,
and capability-sensitive gates retain the requested root spelling for replacement detection.

## Notes

- Every library module is consumed by the CLI — it's the integration point for the entire tool.
- `watch` and `mcp` are the only commands that take over the process (long-running file watcher / stdio server).
- The clap grammar lives in `src/cli.rs`; `main.rs` is dispatch-only. Parser-level tests live in `src/cli.rs`; exit-code tests live in `src/main.rs`.
