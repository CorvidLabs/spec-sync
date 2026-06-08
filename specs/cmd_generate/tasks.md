---
spec: cmd_generate.spec.md
---

## Tasks

- [ ] Add a focused integration test asserting `--model` reaches the resolved provider (currently only provider resolution is covered end-to-end)
- [ ] Add a `--format json` batch-mode test asserting the `requested`/`skipped_already_specced`/`skipped_not_found` shape

## Done

- [x] Initial spec creation with all required sections
- [x] Requirements and acceptance criteria documented
- [x] `--model` flag added (overrides `SPECSYNC_AI_MODEL` env and `aiModel` config)
- [x] Config-triggers-AI gate fix — `resolve_provider_for_generate` enters AI mode from `aiProvider`/`aiCommand` config or `SPECSYNC_AI_PROVIDER`/`SPECSYNC_AI_COMMAND` env, no longer requiring `--provider` (regression: `config_ai_provider_triggers_ai_without_provider_flag`)
- [x] 12-factor env precedence — `SPECSYNC_AI_PROVIDER` outranks configured `aiProvider` (`env_provider_overrides_config_provider`)
- [x] `--provider auto` forces auto-detect even when a provider is configured
- [x] Model precedence wired: `--model` > `SPECSYNC_AI_MODEL` env > `aiModel` config
- [x] `--batch` mode with comma/space expansion, already-specced and not-found reporting
- [x] `--uncovered` accepted as an explicit alias of default behavior
- [x] `--format json` output for both default and batch modes; coverage/validation recomputed post-generation
- [x] Integration coverage for provider resolution (`provider_flag_unknown_provider_errors`, `cli_provider_overrides_config_provider`, `ai_command_overrides_ai_provider`, `auto_detect_defaults_to_local_ollama_without_keys`, etc.)

## Gaps

- No `#[cfg(test)]` unit tests inside `generate.rs`; provider-resolution behavior is exercised only via `tests/integration.rs`
- `--model` propagation into the AI call is not yet asserted by a dedicated end-to-end test

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
