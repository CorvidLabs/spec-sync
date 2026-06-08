---
spec: ai.spec.md
---

## Automated Coverage

### Unit tests (`src/ai.rs`, `cargo test ai::`)

| Group | Tests | What It Covers |
|-------|-------|----------------|
| `safe_truncate` | `safe_truncate_within_limit`, `safe_truncate_exact_limit`, `safe_truncate_truncates_ascii`, `safe_truncate_respects_utf8_boundary`, `safe_truncate_multibyte_sequence`, `safe_truncate_empty_string` | Byte-cap truncation backs up to valid UTF-8 char boundaries |
| `command_for_provider` | `command_for_claude`, `command_for_copilot`, `command_for_cursor_errors`, `command_for_anthropic_errors`, `command_for_custom_errors` | CLI command strings; API providers and custom error out of this path |
| Ollama routing | `ollama_is_an_api_provider`, `ollama_resolves_keyless`, `ollama_host_strips_v1_and_trailing_slash_from_config`, `ollama_host_defaults_to_localhost` | Ollama is an API provider, resolves keyless, host normalization |
| Provider mapping | `openrouter_maps_to_corvid_ai`, `claude_routes_to_anthropic_api_not_cli`, `detection_order_is_api_only_and_ollama_first` | corvid-ai registry names; `claude`→anthropic API; detection order is API-only, Ollama first |
| `ResolvedProvider` Display/Debug | `display_cli_provider`, `display_anthropic_provider`, `display_openai_provider_no_base_url`, `display_openai_provider_with_base_url`, `debug_api_provider_redacts_key` | Display formatting and API-key redaction in `Debug` |
| `postprocess_spec` | `postprocess_strips_markdown_fence`, `postprocess_strips_plain_fence`, `postprocess_strips_md_fence`, `postprocess_no_fence_passthrough`, `postprocess_missing_frontmatter_errors`, `postprocess_leading_whitespace_before_frontmatter` | Code-fence stripping and frontmatter validation |
| `build_prompt` | `build_prompt_contains_module_name`, `build_prompt_truncates_large_files`, `build_prompt_skips_files_over_prompt_limit`, `build_prompt_empty_files` | Prompt assembly, per-file truncation, prompt-size skipping |
| `build_regen_prompt` | `build_regen_prompt_contains_spec_and_requirements`, `build_regen_prompt_no_source_files`, `build_regen_prompt_truncates_large_sources` | Regeneration prompt content and size budget |
| `resolve_ai_provider` | `resolve_with_ai_command_in_config`, `resolve_with_env_var`, `resolve_unknown_provider_errors`, `resolve_cursor_provider_errors`, `resolve_ai_command_returns_cli_string` | Resolution ladder, env override, error cases, compat alias |
| Constants | `constants_are_reasonable` | `MAX_FILE_CHARS`=30K, `MAX_PROMPT_CHARS`=150K, `DEFAULT_AI_TIMEOUT_SECS`=120 |

### Integration tests (`tests/integration.rs`, `cargo test --test integration`)

| Fixture | What It Asserts |
|---------|-----------------|
| `provider_flag_unknown_provider_errors` | Unknown `--provider` name errors |
| `provider_flag_enables_ai` | `--provider` puts `generate` into AI mode |
| `ai_provider_config_field_is_respected` | `aiProvider` config is honored |
| `ai_command_overrides_ai_provider` | `aiCommand` outranks `aiProvider` |
| `cli_provider_overrides_config_provider` | `--provider` outranks config |
| `config_ai_provider_triggers_ai_without_provider_flag` | Configured `aiProvider` enters AI mode with no flag (fails fast on missing key) |
| `env_provider_overrides_config_provider` | `SPECSYNC_AI_PROVIDER` env outranks `aiProvider` config |
| `auto_detect_defaults_to_local_ollama_without_keys` | No keys → defaults to local Ollama, never errors/CLI |
| `anthropic_provider_requires_api_key`, `openai_provider_requires_api_key` | Missing key errors naming the env var |
| `provider_flag_anthropic_requires_api_key`, `provider_flag_openai_requires_api_key` | Same via `--provider` flag |
| `anthropic_api_alias_works`, `openai_api_alias_works` | `anthropic-api` / `openai-api` aliases accepted |
| `ai_api_key_config_field_used_for_anthropic` | `aiApiKey` config field is used for anthropic |
| `unknown_provider_lists_api_options` | Unknown-provider error lists `anthropic` and `openai` among options |

## Manual Testing

- [ ] No keys set, local Ollama running: `specsync generate` defaults to Ollama (`http://localhost:11434`) over `/v1`
- [ ] Exactly one `<PROVIDER>_API_KEY` set: auto-detected and used without prompting
- [ ] Multiple keys set in a terminal: interactive provider + model prompt appears; in CI (non-TTY) the deterministic order is used
- [ ] `--provider claude`: prints the deprecation warning and calls the anthropic API (verify it does NOT spawn `claude -p`)
- [ ] `--model` / `SPECSYNC_AI_MODEL` / `aiModel` override the model in the documented precedence
- [ ] Ollama `-cloud` model tag + `OLLAMA_API_KEY`: routes to `https://ollama.com`

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| No AI provider configured and no keys | Defaults to keyless local Ollama, warns on stderr |
| Key-required provider missing its key (no `aiBaseUrl`) | Fails fast: "requires an API key. Set ENV_VAR or add aiApiKey" |
| Ollama or custom `aiBaseUrl` with no key | Key check skipped; request proceeds (auth errors surface from corvid-ai) |
| `cursor` selected | Error: no CLI pipe mode, with workarounds (or "not installed" if absent) |
| Multibyte content at the truncation boundary | `safe_truncate` backs up to a valid char boundary, never splits a code point |
| Source files exceed 150K total | Remaining files marked "[skipped: prompt size limit]" |
| API key in a resolved provider | Never printed: `Debug` shows `[REDACTED]`; corvid-ai redacts keys in error strings |
| AI returns output without `---` frontmatter | Error: "AI response missing YAML frontmatter delimiters" |
| CLI command times out | Error with the timeout value and a suggestion to raise `aiTimeout` |
| CLI command returns empty stdout | Error: "AI command returned empty output" |

## Reviewer Checklist

- Run `cargo test ai::` (and the relevant `tests/integration.rs` fixtures) before changing `src/ai.rs`.
- If an error or warning string changes, update the matching Edge Case row and test assertion in the same commit.
- Confirm `claude` still routes to the anthropic API (never `claude -p`) and that `Debug`/error output never leaks keys.
- Run the release checks: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
