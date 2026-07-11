---
module: ai
version: 2
status: stable
files:
  - src/ai.rs
db_tables: []
tracks: [19]
depends_on:
  - specs/types/types.spec.md
---

# Ai

## Purpose

Resolves and executes AI providers for spec generation. Supports CLI-based providers (Claude, Ollama, Copilot) and direct API providers (Anthropic, OpenAI). Builds prompts from source code, runs the provider, and post-processes the output to ensure valid spec format.

## Public API

### Exported Enums

| Type | Description |
|------|-------------|
| `ResolvedProvider` | A resolved provider ready to execute: `Cli(String)` (shell out) or `Api(corvid_ai::Settings)` (HTTP via the shared `corvid-ai` client) |

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `resolve_ai_provider` | `config, cli_provider` | `Result<ResolvedProvider, String>` | Resolve which AI provider to use via 5-level priority chain |
| `resolve_ai_command` | `config, cli_provider` | `Result<String, String>` | Legacy alias — resolves provider and returns CLI command string |
| `generate_spec_with_ai` | `module_name, source_files, root, config, provider` | `Result<String, String>` | Generate a spec file by reading source code and calling the AI provider |
| `regenerate_spec_with_ai` | `module_name, spec_path, requirements_path, root, config, provider` | `Result<String, String>` | Regenerate an existing spec using AI when requirements have drifted; reads source files from the spec's frontmatter |

## Invariants

1. Provider resolution order (`flag > env > config`, 12-factor): `--provider` flag > `SPECSYNC_AI_COMMAND` env > `SPECSYNC_AI_PROVIDER` env > `aiCommand` config > `aiProvider` config > auto-detect. `generate` enters AI mode whenever any of these is set (a configured `aiProvider` no longer requires repeating `--provider`); with none set it is template-only. Model precedence is the same shape: `--model` > `SPECSYNC_AI_MODEL` env > `aiModel` config > provider default
2. Auto-detect ladder (shared with fledge), by `<PROVIDER>_API_KEY` presence (no network probe): **none configured → keyless local Ollama** (`http://localhost:11434`); **exactly one configured → use it**; **multiple configured → prompt** for provider + model when interactive (stdin & stderr are TTYs), else fall back to the deterministic order (Ollama, Anthropic, OpenAI, OpenRouter, Gemini, DeepSeek, Groq, Mistral, xAI, Together). A set API key beats unkeyed local Ollama; auto-detect never shells out to a CLI
3. Ollama host for requests: `OLLAMA_HOST` env > `aiBaseUrl` config > `-cloud` routing (model tag contains `-cloud` and `OLLAMA_API_KEY` set ⇒ Ollama Cloud) > `http://localhost:11434`; corvid-ai speaks to it over `/v1`
4. The deprecated `claude` provider routes to the `anthropic` API (with a warning); it no longer shells out to `claude -p`. `copilot`/`cursor` and the explicit trusted `aiCommand` shell escape hatch remain deprecated compatibility paths in 5.0.
5. Source code is capped at 150K characters total and 30K per file to avoid exceeding context windows
6. AI response is post-processed: code fences are stripped, frontmatter delimiters are validated
7. Default timeout is 120 seconds, configurable via `aiTimeout` in config
8. API providers do not require a CLI binary — they use direct HTTP calls via the `corvid-ai` client, which owns the endpoint registry, default models, and `<PROVIDER>_API_KEY` resolution

## Behavioral Examples

### Scenario: Default to local Ollama with no key

- **Given** no provider API key is set and no `aiProvider`/`aiCommand` is configured
- **When** `resolve_ai_provider(config, None)` is called
- **Then** returns `ResolvedProvider::Api` with the `ollama` provider name (keyless, `http://localhost:11434`)

### Scenario: Use Anthropic API key

- **Given** `ANTHROPIC_API_KEY` is set in environment (and no `OLLAMA_API_KEY`)
- **When** `resolve_ai_provider(config, None)` is called
- **Then** returns `ResolvedProvider::Api` with the `anthropic` provider name

### Scenario: Deprecated `claude` routes to Anthropic

- **Given** user passes `--provider claude`
- **When** `resolve_ai_provider(config, Some("claude"))` is called
- **Then** prints a deprecation warning and returns `ResolvedProvider::Api` with the `anthropic` provider name (never shells out to `claude -p`)

### Scenario: Explicit provider override

- **Given** user passes `--provider openai`
- **When** `resolve_ai_provider(config, Some("openai"))` is called
- **Then** returns `ResolvedProvider::Api` with the `openai` provider name (resolved against `OPENAI_API_KEY`)

### Scenario: Multiple keys, non-interactive

- **Given** both `ANTHROPIC_API_KEY` and `OPENAI_API_KEY` are set, no explicit selection, and stdin/stderr are not TTYs
- **When** `resolve_ai_provider(config, None)` is called
- **Then** uses the first match in deterministic order (`anthropic`) without prompting

### Scenario: Generate spec with AI

- **Given** source files for module "auth"
- **When** `generate_spec_with_ai("auth", files, root, config, provider)` is called
- **Then** returns a complete spec markdown string with frontmatter and all required sections

## Error Cases

| Condition | Behavior |
|-----------|----------|
| No AI provider found | Returns descriptive error listing all options |
| Provider binary not installed | Error: "not installed or not on PATH" |
| API key missing | Error: "requires an API key. Set ENV_VAR or add aiApiKey" |
| Cursor selected as provider | Error explaining no CLI pipe mode, with workarounds |
| AI command times out | Error with timeout value and suggestion to increase `aiTimeout` |
| AI returns empty output | Error: "AI command returned empty output" |
| AI response missing frontmatter | Error: "AI response missing YAML frontmatter delimiters" |
| API HTTP error | Error with status code and error message |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| types | `AiProvider`, `SpecSyncConfig` |
| corvid-ai | Shared multi-provider LLM client (`Settings`, `Completion`, `complete`) for all API calls |

### Consumed By

| Module | What is used |
|--------|-------------|
| generator | `generate_spec_with_ai`, `ResolvedProvider` |
| mcp | `resolve_ai_provider` |
| main | `resolve_ai_provider`, `regenerate_spec_with_ai` |

## Change Log

| Date | Change |
|------|--------|
| 2026-07-10 | v2: keep provider-resolution tests clean under current stable Clippy by constructing test configs directly |
| 2026-03-25 | Initial spec |
| 2026-06-07 | Route API providers through the shared `corvid-ai` client; `ResolvedProvider` API variants collapse to `Api(corvid_ai::Settings)` and the per-provider `call_*_api` HTTP code is removed |
| 2026-06-07 | API-first/API-only auto-detection (no CLI shell-out); default to keyless local Ollama when no key is set; `claude` routes to the `anthropic` API; add `openrouter` + `ollama` (HTTP) providers |
| 2026-06-07 | Final resolution ladder (shared with fledge): key-based detection (no probe) — none→keyless local Ollama, one→use it, multiple→prompt (interactive) or deterministic order; Ollama first in that order. `OLLAMA_HOST` + `-cloud` host routing for requests; add `SPECSYNC_AI_PROVIDER` env |
