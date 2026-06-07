---
spec: types.spec.md
---

## Key Decisions

- **Central type definitions**: All shared types live in `types.rs` rather than being scattered across modules. This creates a single source of truth and prevents circular dependencies.
- **Loose string parsing for AI providers**: `AiProvider::from_str_loose()` accepts case-insensitive input with common aliases (e.g., "gh-copilot" → Copilot, "grok"/"x-ai" → XAi, "google" → Gemini, "open-router" → OpenRouter). This makes CLI/config input forgiving without sacrificing type safety internally.
- **API-first, CLI deprecated (4.4.0)**: The AI provider layer was reworked to be API-centric. `detection_order()` is now **API-only** and never shells out — it probes `<PROVIDER>_API_KEY` env vars in a deterministic order (Ollama first, then Anthropic, OpenAi, OpenRouter, Gemini, DeepSeek, Groq, Mistral, XAi, Together). The legacy CLI providers (`claude`, `copilot`, `cursor`) are kept only for explicit selection; `claude` routes to the anthropic API with a deprecation warning.
- **Ollama is an API provider**: As of 4.4.0, Ollama talks to its OpenAI-compatible HTTP endpoint (cloud uses `OLLAMA_API_KEY`; a local server runs keyless). It is no longer a CLI shell-out and `is_api_provider()` returns true for it.
- **Endpoint/default-model registry lives in `corvid-ai`, not here**: `AiProvider::default_model` and `default_base_url` were **removed**. This crate only knows provider identity, the API-key env var name, and (for legacy CLI providers) the binary name. The `corvid-ai` crate — shared with the fledge tool — owns base URLs and default model ids. Keeps this module a thin enum and avoids drift between two endpoint tables.
- **Sensible defaults everywhere**: `SpecSyncConfig::default()` provides working values for all fields so the tool works without any config file.

## Files to Read First

- `src/types.rs` — Single-file module defining all enums, structs, and their `Default`/`Display` implementations.
- `src/ai.rs` — Consumer of `AiProvider`; `resolve_ai_provider` and the Ollama-as-API tests (`ollama_is_an_api_provider`, `ollama_resolves_keyless`) show how these types are used at runtime.
- `src/commands/generate.rs` — Reads `ai_model`/`ai_provider` and applies the `flag > env > config` precedence.

## Current Status

Fully implemented and stable. The types module is consumed by every other module in the project, so changes here have the widest blast radius. The 4.4.0 AI rework (OpenRouter + the OpenAI-compatible family, Ollama-as-API, removal of `default_model`/`default_base_url`, API-only `detection_order`) is complete.

## Notes

- `ModuleDefinition` in config allows users to explicitly define modules with their source files, overriding auto-detection — the escape hatch for non-standard layouts.
- The `OutputFormat` enum (Text, Json, Markdown, Github, Table, Csv) determines CLI output formatting across all reporting commands.
- `SpecSyncConfig` carries `ai_model` and `ai_base_url`; `ai_base_url` overrides the OpenAI-compatible endpoint for local proxies.
