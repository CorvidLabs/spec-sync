---
spec: cli_args.spec.md
---

## Tasks

- [ ] Add an inline parse test asserting `generate --model <id>` lands in `Command::Generate { model: Some(..) }`
- [ ] Add a parse test for the `auto` sentinel on `--provider`

## Done

- [x] Top-level `Cli` parser with global flags (`--strict`, `--require-coverage`, `--root`, `--format`, `--json`, `--enforcement`, `--exclude-status`, `--only-status`)
- [x] Full `Command` subcommand enum (Check … Lifecycle), `HooksAction`, and `LifecycleAction`
- [x] Inline `#[cfg(test)]` parser tests in `cli.rs` (no-subcommand default, global-flag ordering, json format, check flag collection, stale threshold default/override, comma-split exclude-status, unknown-subcommand/non-numeric rejection)
- [x] **4.4.0 AI rework — DONE:**
  - [x] Add `--model <MODEL>` flag to `Generate` (overrides `SPECSYNC_AI_MODEL` env and `aiModel` config)
  - [x] Reword `--provider` help to list the API providers (anthropic, openai, openrouter, gemini, deepseek, groq, mistral, xai, together, ollama) + deprecated claude/copilot, and document the `auto` sentinel
- [x] Integration coverage for `--provider` behavior (`provider_flag_unknown_provider_errors`, `provider_flag_enables_ai`, `cli_provider_overrides_config_provider`, `env_provider_overrides_config_provider`)
- [x] Complete `ChangeAction` namespace and inline parser coverage for SDD creation scope

## Gaps

- No inline parse test specifically pins the `Generate { provider, model }` fields (covered end-to-end via integration tests, but not at the parser level)

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
