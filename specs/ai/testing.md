---
spec: ai.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/ai.rs` | cargo test ai:: | `safe_truncate_within_limit`, `safe_truncate_exact_limit`, `safe_truncate_truncates_ascii`, `safe_truncate_respects_utf8_boundary`, `safe_truncate_multibyte_sequence`, `safe_truncate_empty_string` |
| `tests/integration.rs` | cargo test --test integration provider_flag_unknown_provider_errors | End-to-end fixture: `provider_flag_unknown_provider_errors` |
| `tests/integration.rs` | cargo test --test integration provider_flag_enables_ai | End-to-end fixture: `provider_flag_enables_ai` |
| `tests/integration.rs` | cargo test --test integration ai_provider_config_field_is_respected | End-to-end fixture: `ai_provider_config_field_is_respected` |
| `tests/integration.rs` | cargo test --test integration ai_command_overrides_ai_provider | End-to-end fixture: `ai_command_overrides_ai_provider` |
| `tests/integration.rs` | cargo test --test integration cli_provider_overrides_config_provider | End-to-end fixture: `cli_provider_overrides_config_provider` |
| `tests/integration.rs` | cargo test --test integration ai_model_config_used_with_ollama_provider | End-to-end fixture: `ai_model_config_used_with_ollama_provider` |
| `tests/integration.rs` | cargo test --test integration anthropic_provider_requires_api_key | End-to-end fixture: `anthropic_provider_requires_api_key` |
| `tests/integration.rs` | cargo test --test integration openai_provider_requires_api_key | End-to-end fixture: `openai_provider_requires_api_key` |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Auto-detect Claude CLI | `claude` binary is on PATH and no config overrides | `resolve_ai_provider(config, None)` is called | returns `ResolvedProvider::Cli("claude -p --output-format text")` |
| Use Anthropic API key | `ANTHROPIC_API_KEY` is set in environment, no CLI providers installed | `resolve_ai_provider(config, None)` is called | returns `ResolvedProvider::AnthropicApi` with the key and default model |
| Explicit provider override | user passes `--provider openai` | `resolve_ai_provider(config, Some("openai"))` is called | returns `ResolvedProvider::OpenAiApi` using OPENAI_API_KEY |
| Generate spec with AI | source files for module "auth" | `generate_spec_with_ai("auth", files, root, config, provider)` is called | returns a complete spec markdown string with frontmatter and all required sections |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| No AI provider found | Returns descriptive error listing all options | Keep or add a focused assertion before changing this behavior |
| Provider binary not installed | Error: "not installed or not on PATH" | Keep or add a focused assertion before changing this behavior |
| API key missing | Error: "requires an API key. Set ENV_VAR or add aiApiKey" | Keep or add a focused assertion before changing this behavior |
| Cursor selected as provider | Error explaining no CLI pipe mode, with workarounds | Keep or add a focused assertion before changing this behavior |
| AI command times out | Error with timeout value and suggestion to increase `aiTimeout` | Keep or add a focused assertion before changing this behavior |
| AI returns empty output | Error: "AI command returned empty output" | Keep or add a focused assertion before changing this behavior |
| AI response missing frontmatter | Error: "AI response missing YAML frontmatter delimiters" | Keep or add a focused assertion before changing this behavior |
| API HTTP error | Error with status code and error message | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/ai.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
