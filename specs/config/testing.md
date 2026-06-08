---
spec: config.spec.md
---

## Automated Coverage

Unit tests live in the `#[cfg(test)]` module at the bottom of `src/config.rs`. Run with `cargo test config::`.

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| TOML scalar/array/bool parsing | cargo test config:: | `test_parse_toml_string_quoted`, `test_parse_toml_string_unquoted`, `test_parse_toml_string_empty_quotes`, `test_parse_toml_string_with_whitespace`, `test_parse_toml_string_array_basic`, `test_parse_toml_string_array_single`, `test_parse_toml_string_array_empty`, `test_parse_toml_bool_true_variants`, `test_parse_toml_bool_false_variants` |
| Load + format precedence | cargo test config:: | `test_load_config_json`, `test_load_config_toml`, `test_load_config_toml_takes_priority_over_json`, `test_load_config_v4_toml_takes_priority`, `test_load_config_no_config_file`, `test_load_config_malformed_json_returns_defaults`, `test_load_config_json_without_source_dirs_auto_detects`, `test_toml_full_config`, `test_toml_comments_and_blank_lines`, `test_toml_without_source_dirs_auto_detects` |
| `config.local.toml` AI overrides | cargo test config:: | `test_local_config_overrides_ai_provider`, `test_local_config_overrides_ai_command`, `test_local_config_missing_is_fine`, `test_local_config_works_with_legacy_json`, `test_local_config_strips_inline_comments`, `test_strip_inline_comment_preserves_hash_in_quotes` |
| Source-dir detection | cargo test config:: | `test_detect_source_dirs_empty_project`, `test_detect_source_dirs_with_src_dir`, `test_detect_source_dirs_ignores_node_modules`, `test_detect_source_dirs_root_source_files` |
| `config_to_toml` round-trip & companions | cargo test config:: | `test_config_to_toml_roundtrips_companions`, `test_toml_companions_design_enabled`, `test_toml_companions_design_default_false` |
| Schema-pattern default | cargo test config:: | `test_default_schema_pattern_matches_create_table`, `test_default_schema_pattern_captures_table_name` |
| AI config field (e2e) | cargo test --test integration ai_provider_config_field_is_respected | End-to-end fixture: `ai_provider_config_field_is_respected` |
| AI key from config (e2e) | cargo test --test integration ai_api_key_config_field_used_for_anthropic | End-to-end fixture: `ai_api_key_config_field_used_for_anthropic` |
| Config triggers AI (e2e) | cargo test --test integration config_ai_provider_triggers_ai_without_provider_flag | End-to-end fixture: `config_ai_provider_triggers_ai_without_provider_flag` |
| Env over config (e2e) | cargo test --test integration env_provider_overrides_config_provider | End-to-end fixture: `env_provider_overrides_config_provider` |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Load JSON config | a `specsync.json` exists at project root with `"specsDir": "docs/specs"` | `load_config(root)` is called | returns a config with `specs_dir = "docs/specs"` |
| No config file | no config file exists | `load_config(root)` is called | returns default config with auto-detected source dirs |
| v4 TOML wins | `.specsync/config.toml` and legacy `specsync.json` both present | `load_config(root)` is called | v4 TOML values win over the legacy root file |
| Local override merge | committed `config.toml` sets `ai_provider = "claude"`, `config.local.toml` sets `ai_provider = "ollama"` and `ai_model = "llama3"` | `load_config(root)` is called | `ai_provider = Ollama`, `ai_model = "llama3"`, non-AI fields unchanged |
| API key not serialized | a config with `ai_api_key` set | `config_to_toml(&config)` is called | output omits `ai_api_key` and a warning is printed to stderr |
| Auto-detect source dirs | a project root with `src/` and `lib/` containing `.rs` files | `detect_source_dirs(root)` is called | returns `["lib", "src"]` (sorted alphabetically) |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Config file unreadable | Falls back to `SpecSyncConfig::default()` | Keep or add a focused assertion before changing this behavior |
| Malformed JSON config | Prints warning to stderr, falls back to defaults | `test_load_config_malformed_json_returns_defaults` |
| Empty project root | Returns `["src"]` as source dirs | `test_detect_source_dirs_empty_project` |
| `config.local.toml` only applies AI keys | Section keys and unknown top-level keys are skipped/warned | `test_local_config_overrides_ai_provider` + manual unknown-key check |
| `#` inside a quoted local-config value | Not treated as a comment | `test_strip_inline_comment_preserves_hash_in_quotes` |
| `ai_api_key` round-trip | Never written back by `config_to_toml` | Add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/config.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
