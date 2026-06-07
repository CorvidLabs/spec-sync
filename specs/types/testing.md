---
spec: types.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/types.rs` | cargo test types | No inline tests found; add focused coverage for `AiProvider`, `Language`, `OutputFormat`, `ExportLevel` before risky changes |
| `tests/integration.rs` | cargo test --test integration multi_lang_typescript | End-to-end fixture: `multi_lang_typescript` |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Parse AI provider from string | the string "anthropic-api" | `AiProvider::from_str_loose("anthropic-api")` is called | returns `Some(AiProvider::Anthropic)` |
| Detect language from file extension | a file with extension "tsx" | `Language::from_extension("tsx")` is called | returns `Some(Language::TypeScript)` |
| Detect Ruby from file extension | a file with extension "rb" | `Language::from_extension("rb")` is called | returns `Some(Language::Ruby)` |
| Unknown file extension | a file with extension "haskell" | `Language::from_extension("haskell")` is called | returns `None` |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Unknown provider string | `AiProvider::from_str_loose` returns `None` | Keep or add a focused assertion before changing this behavior |
| Unsupported file extension | `Language::from_extension` returns `None` | Keep or add a focused assertion before changing this behavior |
| Invalid JSON config | `SpecSyncConfig` deserialization fails at the caller level | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/types.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `./target/release/specsync score --all`.
