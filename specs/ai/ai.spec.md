---
module: ai
version: 1
status: stable
files:
  - src/ai.rs
db_tables: []
tracks: [110]
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
| `ResolvedProvider` | A resolved provider ready to execute: `Cli(String)`, `AnthropicApi{...}`, or `OpenAiApi{...}` |

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `resolve_ai_provider` | `config, cli_provider` | `Result<ResolvedProvider, String>` | Resolve which AI provider to use via 5-level priority chain |
| `resolve_ai_command` | `config, cli_provider` | `Result<String, String>` | Legacy alias — resolves provider and returns CLI command string |
| `generate_spec_with_ai` | `module_name, source_files, root, config, provider` | `Result<String, String>` | Generate a spec file by reading source code and calling the AI provider |
| `regenerate_spec_with_ai` | `module_name, spec_path, requirements_path, root, config, provider` | `Result<String, String>` | Regenerate an existing spec using AI when requirements have drifted; reads source files from the spec's frontmatter |

## Invariants

1. Provider resolution order: CLI `--provider` flag > `aiCommand` config > `aiProvider` config > `SPECSYNC_AI_COMMAND` env var > auto-detect
2. Auto-detection checks CLI providers first (by attempting to run `<binary> --version` via OS-level execvp), then API providers (by env var presence)
3. Source code is capped at 150K characters total and 30K per file to avoid exceeding context windows
4. AI response is post-processed: code fences are stripped, frontmatter delimiters are validated
5. Default timeout is 120 seconds, configurable via `aiTimeout` in config
6. Cursor provider explicitly errors — it has no stdin/stdout pipe mode
7. API providers (Anthropic, OpenAI) do not require a CLI binary — they use direct HTTP calls via `ureq`
8. CLI execution streams stdout lines to stderr in real time for live progress feedback

## Behavioral Examples

### Scenario: Auto-detect Claude CLI

- **Given** `claude` binary is on PATH and no config overrides
- **When** `resolve_ai_provider(config, None)` is called
- **Then** returns `ResolvedProvider::Cli("claude -p --output-format text")`

### Scenario: Use Anthropic API key

- **Given** `ANTHROPIC_API_KEY` is set in environment, no CLI providers installed
- **When** `resolve_ai_provider(config, None)` is called
- **Then** returns `ResolvedProvider::AnthropicApi` with the key and default model

### Scenario: Explicit provider override

- **Given** user passes `--provider openai`
- **When** `resolve_ai_provider(config, Some("openai"))` is called
- **Then** returns `ResolvedProvider::OpenAiApi` using OPENAI_API_KEY

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
| ureq | HTTP client for Anthropic and OpenAI API calls |

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
