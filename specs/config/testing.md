---
spec: config.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/config.rs` | cargo test config:: | `test_parse_toml_string_quoted`, `test_parse_toml_string_unquoted`, `test_parse_toml_string_empty_quotes`, `test_parse_toml_string_with_whitespace`, `test_parse_toml_string_array_basic`, `test_parse_toml_string_array_single` |
| `tests/integration.rs` | cargo test --test integration init_creates_config_file | End-to-end fixture: `init_creates_config_file` |
| `tests/integration.rs` | cargo test --test integration init_does_not_overwrite_existing_config | End-to-end fixture: `init_does_not_overwrite_existing_config` |
| `tests/integration.rs` | cargo test --test integration ai_provider_config_field_is_respected | End-to-end fixture: `ai_provider_config_field_is_respected` |
| `tests/integration.rs` | cargo test --test integration cli_provider_overrides_config_provider | End-to-end fixture: `cli_provider_overrides_config_provider` |
| `tests/integration.rs` | cargo test --test integration ai_model_config_used_with_ollama_provider | End-to-end fixture: `ai_model_config_used_with_ollama_provider` |
| `tests/integration.rs` | cargo test --test integration ai_api_key_config_field_used_for_anthropic | End-to-end fixture: `ai_api_key_config_field_used_for_anthropic` |
| `tests/integration.rs` | cargo test --test integration check_works_without_config_file | End-to-end fixture: `check_works_without_config_file` |
| `tests/integration.rs` | cargo test --test integration mcp_tool_init_creates_config | End-to-end fixture: `mcp_tool_init_creates_config` |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Load JSON config | a `specsync.json` exists at project root with `"specsDir": "docs/specs"` | `load_config(root)` is called | returns a config with `specs_dir = "docs/specs"` |
| No config file | no `specsync.json` or `.specsync.toml` exists | `load_config(root)` is called | returns default config with auto-detected source dirs |
| Auto-detect source dirs | a project root with `src/` and `lib/` containing `.rs` files | `detect_source_dirs(root)` is called | returns `["lib", "src"]` (sorted alphabetically) |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Config file unreadable | Falls back to `SpecSyncConfig::default()` | Keep or add a focused assertion before changing this behavior |
| Malformed JSON config | Prints warning to stderr, falls back to defaults | Keep or add a focused assertion before changing this behavior |
| Empty project root | Returns `["src"]` as source dirs | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/config.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
