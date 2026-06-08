---
spec: types.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/types.rs` | cargo test types | No inline `#[cfg(test)]` tests in this file; `AiProvider`/`Language`/`SpecStatus` are exercised indirectly. Add focused unit coverage before risky changes. |
| `src/ai.rs` | cargo test --lib ollama_is_an_api_provider | Asserts `AiProvider::Ollama.is_api_provider()` is true (Ollama reclassified as API in 4.4.0) |
| `src/ai.rs` | cargo test --lib ollama_resolves_keyless | Local Ollama resolves without an API key |
| `tests/integration.rs` | cargo test --test integration provider_flag_unknown_provider_errors | Unknown `--provider` value errors with "Unknown provider" |
| `tests/integration.rs` | cargo test --test integration auto_detect_defaults_to_local_ollama_without_keys | With no `<PROVIDER>_API_KEY` set, auto-detect falls back to local Ollama (detection_order is Ollama-first) |
| `tests/integration.rs` | cargo test --test integration generate_with_multiple_languages | End-to-end multi-language export/coverage exercising the `Language` enum |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Parse AI provider from string | the string "anthropic-api" | `AiProvider::from_str_loose("anthropic-api")` is called | returns `Some(AiProvider::Anthropic)` |
| Parse aliased provider | the string "grok" | `AiProvider::from_str_loose("grok")` is called | returns `Some(AiProvider::XAi)` |
| Ollama is an API provider | the variant `AiProvider::Ollama` | `is_api_provider()` is called | returns `true` |
| Detection order is API-only | `AiProvider::detection_order()` | inspect the slice | first element is `Ollama`; no Claude/Copilot/Cursor present |
| Detect language from file extension | a file with extension "tsx" | `Language::from_extension("tsx")` is called | returns `Some(Language::TypeScript)` |
| Detect Ruby from file extension | a file with extension "rb" | `Language::from_extension("rb")` is called | returns `Some(Language::Ruby)` |
| Unknown file extension | a file with extension "haskell" | `Language::from_extension("haskell")` is called | returns `None` |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Unknown provider string | `AiProvider::from_str_loose` returns `None` | Keep or add a focused assertion before changing this behavior |
| Ollama classified as API | `is_api_provider()` true and `detection_order` is API-only/Ollama-first | Covered by `ollama_is_an_api_provider`; do not regress to a CLI shell-out |
| No default_model/default_base_url on AiProvider | Endpoint/default-model registry stays in `corvid-ai`, not `types.rs` | Do not re-add these methods here |
| Unsupported file extension | `Language::from_extension` returns `None` | Keep or add a focused assertion before changing this behavior |
| Invalid JSON config | `SpecSyncConfig` deserialization fails at the caller level | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/types.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `./target/release/specsync score --all`.
