---
spec: cli_args.spec.md
---

## User Stories

- As a user of the `specsync` binary, I want every subcommand, flag, and global option declared in one Clap parser so that `--help` and argument validation are consistent across the CLI
- As a CI operator, I want global flags (`--strict`, `--require-coverage`, `--enforcement`, `--format`/`--json`, `--root`, `--exclude-status`/`--only-status`) to work regardless of where they appear relative to the subcommand
- As a user generating specs with AI, I want a `--provider` flag and a `--model` flag on `generate` so that I can pick a provider and model id from the command line without editing config
- As a developer, I want invalid arguments rejected by Clap with usage help so that mistakes fail fast and visibly

## Acceptance Criteria

- `Cli` exposes global flags: `--strict`, `--require-coverage <N>`, `--root <path>`, `--format <text|json|markdown|github|table|csv>`, `--json`, `--enforcement <warn|enforce-new|strict>`, `--exclude-status <...>`, `--only-status <...>` — all `global = true`
- `--json` is shorthand for `--format json`; default format is `text`
- `Command` enum covers all current subcommands (Check, Coverage, Generate, Init, Score, Watch, Mcp, AddSpec, Scaffold, InitRegistry, Resolve, Diff, Hooks, Compact, ArchiveTasks, View, Merge, Issues, New, Wizard, Deps, Import, Stale, Report, Comment, Rules, Changelog, Rehash, Migrate, Lifecycle)
- The `Generate` command exposes `--provider <PROVIDER>` (or `auto`) and `--model <MODEL>`, plus `--uncovered` and `--batch <MODULE...>`
- `--provider` accepts the API provider names (anthropic, openai, openrouter, gemini, deepseek, groq, mistral, xai, together, ollama) plus the deprecated `claude`/`copilot`; the actual resolution/precedence lives in `generate.rs`, not the parser
- Running `specsync` with no subcommand yields `Cli.command == None` (main.rs defaults to Check behavior)
- `HooksAction` (Install/Uninstall/Status) and `LifecycleAction` (Promote/Demote/Set/Status/History/Guard/AutoPromote/Enforce) are declared as sub-subcommands
- Invalid enum values for `--format`/`--enforcement` and unknown subcommands are rejected by Clap with usage help

## Constraints

- This module only *declares* the argument surface — flag/env/config precedence and command behavior live in `commands/`, `ai.rs`, and `main.rs`. Keep parsing free of business logic.
- `--provider`/`--model` are `Option<String>` (loose strings); validation is deferred to `AiProvider::from_str_loose` and `resolve_ai_provider` so error messages can list available providers.
- Must compile on MSRV 1.89.

## Out of Scope

- Resolving the effective AI provider/model (precedence `flag > env > config`) — handled in `src/commands/generate.rs`
- Executing subcommands (handled by `src/commands/*` and `src/main.rs`)
- GUI or web interface; interactive prompts beyond the `wizard` subcommand
