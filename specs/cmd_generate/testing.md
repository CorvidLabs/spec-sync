---
spec: cmd_generate.spec.md
---

## Automated Coverage

`src/commands/generate.rs` has no inline `#[cfg(test)]` tests; coverage is end-to-end via `tests/integration.rs`. Provider-resolution semantics also have deterministic unit tests in `src/ai.rs` (e.g. `ollama_is_an_api_provider`, `ollama_resolves_keyless`).

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| Scaffolding (templates) | cargo test --test integration generate_creates_spec_for_unspecced_module | `generate_creates_spec_for_unspecced_module` |
| No-op when covered | cargo test --test integration generate_no_op_when_fully_covered | `generate_no_op_when_fully_covered` |
| Multi-language | cargo test --test integration generate_with_multiple_languages | `generate_with_multiple_languages` |
| `--uncovered` alias | cargo test --test integration generate_uncovered_flag_accepted | `generate_uncovered_flag_accepted` |
| `--batch` empty list | cargo test --test integration generate_batch_empty_list_skips_gracefully | `generate_batch_empty_list_skips_gracefully` |
| Companion files | cargo test --test integration generate_creates_companion_files | `generate_creates_companion_files` |
| design.md when enabled | cargo test --test integration generate_creates_design_md_when_enabled | `generate_creates_design_md_when_enabled` |
| Unknown provider errors | cargo test --test integration provider_flag_unknown_provider_errors | `provider_flag_unknown_provider_errors` (stderr "Unknown provider", exit 1) |
| `--provider` enables AI | cargo test --test integration provider_flag_enables_ai | `provider_flag_enables_ai` |
| Config triggers AI | cargo test --test integration config_ai_provider_triggers_ai_without_provider_flag | `config_ai_provider_triggers_ai_without_provider_flag` (no flag, fails on missing `ANTHROPIC_API_KEY`) |
| Env > config precedence | cargo test --test integration env_provider_overrides_config_provider | `env_provider_overrides_config_provider` (`SPECSYNC_AI_PROVIDER` outranks `aiProvider`) |
| Flag > config precedence | cargo test --test integration cli_provider_overrides_config_provider | `cli_provider_overrides_config_provider` |
| `aiCommand` > `aiProvider` | cargo test --test integration ai_command_overrides_ai_provider | `ai_command_overrides_ai_provider` (falls back to template, stderr shows AI attempted) |
| `--provider auto` honors config | cargo test --test integration ai_provider_config_field_is_respected | `ai_provider_config_field_is_respected` |
| Keyless auto-detect → Ollama | cargo test --test integration auto_detect_defaults_to_local_ollama_without_keys | `auto_detect_defaults_to_local_ollama_without_keys` |
| API key required | cargo test --test integration anthropic_provider_requires_api_key | `anthropic_provider_requires_api_key`, `openai_provider_requires_api_key` |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Template-only default | no provider configured anywhere, 1 unspecced module | `specsync generate` | header "Generating Specs", template spec written, no AI call |
| Config-triggered AI | `aiProvider: "anthropic"` in `specsync.json`, no `--provider`, `ANTHROPIC_API_KEY` unset | `specsync generate` | AI mode selected; fails fast on missing key (exit 1, stderr names `ANTHROPIC_API_KEY`) |
| Env overrides config | config `aiProvider: "ollama"`, env `SPECSYNC_AI_PROVIDER=anthropic`, no key | `specsync generate` | resolves anthropic (env wins), fails on missing `ANTHROPIC_API_KEY` |
| `--provider auto` | config sets an installed-only/cursor provider | `specsync generate --provider auto` | auto-detect path runs (does not silently ignore the configured intent) |
| Batch subset | modules `a` (specced), `b` (unspecced), `c` (absent) | `specsync generate --batch a,b,c` | generates `b`; reports `a` skipped-already-specced, `c` not-found |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Unknown provider | Prints error, exits 1 | `provider_flag_unknown_provider_errors` |
| Configured provider, no flag | Enters AI mode (not template-only) | `config_ai_provider_triggers_ai_without_provider_flag` |
| `SPECSYNC_AI_PROVIDER` vs config | Env wins | `env_provider_overrides_config_provider` |
| AI fails for one module | Falls back to template, continues | `ai_command_overrides_ai_provider` |
| No provider anywhere | Template-only generation | `generate_creates_spec_for_unspecced_module` |
| All modules already specced | No specs generated, full-coverage line shown | `generate_no_op_when_fully_covered` |

## Reviewer Checklist

- Run `cargo run -- generate --help` and confirm it still names `--provider`, `--model`, `--uncovered`, `--batch` with current help text.
- When touching `resolve_provider_for_generate`, re-run the precedence trio: `config_ai_provider_triggers_ai_without_provider_flag`, `env_provider_overrides_config_provider`, `cli_provider_overrides_config_provider`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
