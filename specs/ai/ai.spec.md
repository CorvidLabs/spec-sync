---
module: ai
version: 1
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

1. Provider resolution order: `--provider` flag > `aiCommand` config > `aiProvider` config > `SPECSYNC_AI_PROVIDER` env > `SPECSYNC_AI_COMMAND` env > auto-detect
2. Auto-detect tries **Ollama first when usable** — `OLLAMA_API_KEY` set (cloud) or a local daemon answering `GET {host}/api/tags` within a 2s probe — so a running Ollama wins over a present API key. The Ollama host is `OLLAMA_HOST` env > `aiBaseUrl` config > `-cloud`-routing > `http://localhost:11434`
3. Otherwise auto-detect is **API-first**: the first provider in `detection_order` with a `<PROVIDER>_API_KEY` set wins. It never shells out to a CLI
4. When nothing is usable, resolution still defaults to Ollama (so the failure is a clear "couldn't reach Ollama" rather than "no provider")
5. The deprecated `claude` provider routes to the `anthropic` API (with a warning); it no longer shells out to `claude -p`. `copilot`/`cursor` are deprecated; `aiCommand` remains the explicit, trusted shell escape hatch
6. Source code is capped at 150K characters total and 30K per file to avoid exceeding context windows
7. AI response is post-processed: code fences are stripped, frontmatter delimiters are validated
8. Default timeout is 120 seconds, configurable via `aiTimeout` in config
9. API providers do not require a CLI binary — they use direct HTTP calls via the `corvid-ai` client, which owns the endpoint registry, default models, and `<PROVIDER>_API_KEY` resolution

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
| 2026-03-25 | Initial spec |
| 2026-06-07 | Route API providers through the shared `corvid-ai` client; `ResolvedProvider` API variants collapse to `Api(corvid_ai::Settings)` and the per-provider `call_*_api` HTTP code is removed |
| 2026-06-07 | API-first/API-only auto-detection (no CLI shell-out); default to keyless local Ollama when no key is set; `claude` routes to the `anthropic` API; add `openrouter` + `ollama` (HTTP) providers |
| 2026-06-07 | Ollama-first auto-detect via a 2s `/api/tags` reachability probe (a running Ollama wins over a present API key); `OLLAMA_HOST` + `-cloud` host routing; add `SPECSYNC_AI_PROVIDER` env. Aligns with fledge's resolution ladder |
