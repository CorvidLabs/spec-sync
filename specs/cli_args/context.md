---
spec: cli_args.spec.md
---

## Key Decisions

- **Declaration-only module**: `cli.rs` defines the Clap `Cli`/`Command`/`HooksAction`/`LifecycleAction` types and nothing else. Dispatch and behavior live in `src/main.rs` and `src/commands/*`. Keeping parsing separate from logic makes the CLI surface easy to audit and unit-test in isolation.
- **Global flags via `global = true`**: shared flags are attached to the top-level `Cli` so they parse before or after the subcommand. `--json` is a convenience alias for `--format json`.
- **Loose provider/model strings**: `--provider` and `--model` are `Option<String>`, not typed enums. Validation is deferred so `resolve_ai_provider` can emit a helpful error that lists the available providers, and so `auto` can act as a sentinel forcing auto-detection.
- **Provider/model precedence is NOT here (4.4.0)**: the `flag > env > config` resolution (`--provider` > `SPECSYNC_AI_PROVIDER` > `aiProvider`; `--model` > `SPECSYNC_AI_MODEL` > `aiModel`) lives in `src/commands/generate.rs::resolve_provider_for_generate`. The parser just captures the raw flag values.

## Files to Read First

- `src/cli.rs` — the Clap parser: all subcommands, flags, and the inline `#[cfg(test)]` parser tests.
- `src/commands/generate.rs` — consumes `Generate { provider, model, .. }` and applies the flag/env/config precedence.
- `src/main.rs` — `Cli::parse()` entry point and command dispatch (no-subcommand defaults to Check).

## Current Status

Fully implemented and stable. Updated for the 4.4.0 AI rework: the `Generate` command gained a `--model` flag and the `--provider` help text now lists the API provider family (anthropic, openai, openrouter, gemini, deepseek, groq, mistral, xai, together, ollama) plus the deprecated `claude`/`copilot`.

## Notes

- This module is part of the command layer — it orchestrates nothing itself; it is consumed by `main.rs` (via `Cli::parse()`) and the per-command modules.
- The deprecated `claude`/`copilot`/`cursor` providers are still accepted as `--provider` strings for backward compatibility; `claude` routes to the anthropic API with a warning downstream.
