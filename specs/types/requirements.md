---
spec: types.spec.md
---

## User Stories

- As a spec-sync contributor, I want all shared types defined in one module so that other modules import from a single source of truth
- As a developer adding a new AI provider, I want the AiProvider enum to expose clear, minimal helpers (`binary_name`, `is_api_provider`, `api_key_env_var`, `from_str_loose`, `detection_order`) so that adding an API provider is a small, local change and the endpoint/default-model registry stays out of this crate
- As a developer adding a new language, I want the Language enum to map extensions to languages and provide test patterns so that language support is consistent
- As an integrator consuming spec-sync output, I want ValidationResult and CoverageReport to be well-structured so that I can process results programmatically

## Acceptance Criteria

- AiProvider enum includes, in declaration order: Claude, Cursor, Copilot, Ollama, Anthropic, OpenAi, OpenRouter, Gemini, DeepSeek, Groq, Mistral, XAi, Together, Custom
- `AiProvider::is_api_provider()` is true for anthropic, openai, openrouter, gemini, deepseek, groq, mistral, xai, together, and **ollama** (Ollama is an OpenAI-compatible HTTP API provider, not a CLI shell-out)
- `AiProvider::from_str_loose` is case-insensitive and supports common aliases ("gh-copilot" → Copilot, "grok"/"x-ai" → XAi, "google" → Gemini, "open-router" → OpenRouter, "together-ai" → Together)
- `AiProvider::api_key_env_var` returns the `<PROVIDER>_API_KEY` name for every API provider, including `OLLAMA_API_KEY` and `OPENROUTER_API_KEY`
- `AiProvider::detection_order` is **API-only** (never returns CLI providers), ordered Ollama, Anthropic, OpenAi, OpenRouter, Gemini, DeepSeek, Groq, Mistral, XAi, Together
- AiProvider does NOT carry endpoint or default-model knowledge — `default_model`/`default_base_url` were removed; the `corvid-ai` crate owns that registry now
- The deprecated CLI variants (Claude, Copilot, Cursor) remain in the enum but are reachable only by explicit selection; `claude` routes to the anthropic API with a deprecation warning
- Language enum covers all 12 supported languages (incl. Yaml) with correct extension mappings
- `Language::from_extension` returns None for unsupported extensions (no panic)
- `Language::test_patterns` returns language-appropriate test file patterns
- OutputFormat enum includes Text (default), Json, Markdown, Github, Table, and Csv variants
- ExportLevel enum has Type (top-level declarations only) and Member (all public symbols, default) variants
- `SpecSyncConfig::default()` provides sensible defaults for all fields (incl. `ai_model`, `ai_base_url`)
- `ValidationResult::new()` initializes with empty error/warning/fix vectors
- All config types derive `serde::Deserialize` for JSON config parsing where needed

## Constraints

- Types module must have zero dependencies on other spec-sync modules (it's the foundation)
- AI endpoint URLs and default model ids must NOT live here — that registry belongs to `corvid-ai` (the crate shared with the fledge tool); this module only knows provider identity, API-key env var, and CLI binary name
- Default implementations must produce valid, usable values
- MSRV 1.89 — no APIs newer than the pinned toolchain

## Out of Scope

- AI endpoint resolution and default model selection (owned by `corvid-ai`)
- Actually calling any provider (lives in `src/ai.rs`)
- Runtime type validation (types are validated at compile time via Rust's type system)
- Backwards-compatible type versioning
