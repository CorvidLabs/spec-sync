---
spec: types.spec.md
---

## Tasks

- [ ] Add builder pattern for `SpecSyncConfig` to simplify test construction
- [ ] Consider splitting large enums into sub-modules if type count grows significantly
- [ ] Add inline `#[cfg(test)]` unit tests in `types.rs` for `AiProvider`/`Language`/`SpecStatus` (currently only covered indirectly via `ai.rs` and integration tests)

## Done

- [x] Core enums: AiProvider, Language, OutputFormat, ExportLevel, ParseMode, EnforcementMode
- [x] Core structs: Frontmatter, ValidationResult, CoverageReport, SpecSyncConfig
- [x] `SpecStatus` lifecycle enum (draft→review→active→stable→deprecated→archived) with ordinal/next/prev/valid_transitions/can_transition_to
- [x] Loose string parsing for AiProvider with aliases
- [x] Language detection from file extensions (12 languages incl. Yaml)
- [x] Default implementations for all config types
- [x] ModuleDefinition for explicit module configuration
- [x] RegistryEntry for cross-project registry
- [x] CustomRule / CustomRuleType / RuleSeverity / RuleFilter for declarative validation rules
- [x] LifecycleConfig / TransitionGuard / CompanionConfig / GitHubConfig config structs
- [x] **4.4.0 AI rework — DONE:**
  - [x] Add `OpenRouter`, `Gemini`, `DeepSeek`, `Groq`, `Mistral`, `XAi`, `Together` API providers
  - [x] Reclassify `Ollama` as an API provider (OpenAI-compatible HTTP, `OLLAMA_API_KEY`) — no CLI shell-out
  - [x] Remove `AiProvider::default_model` and `default_base_url` (endpoint/default-model registry now owned by `corvid-ai`)
  - [x] Make `detection_order()` API-only, Ollama-first, with no CLI providers
  - [x] Add `OLLAMA_API_KEY` / `OPENROUTER_API_KEY` to `api_key_env_var`
  - [x] Mark `claude`/`copilot`/`cursor` as deprecated CLI variants (`claude` routes to anthropic API + warning)
  - [x] Add `ai_model` / `ai_base_url` fields to `SpecSyncConfig` (consumed by the `--model` flag in `cli_args`)

## Gaps

- `version` field in `Frontmatter` is typed `Option<String>` — no semver validation at the type level
- `AiProvider::Custom` carries no command itself; the command comes from `SpecSyncConfig::ai_command`

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
