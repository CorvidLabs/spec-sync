---
spec: ai.spec.md
---

## User Stories

- As a developer with no AI setup, I want spec-sync to default to a local, keyless Ollama server so that I can generate specs with zero configuration and my code never leaves my machine
- As a developer with a single provider API key set, I want spec-sync to auto-detect and use it so that I don't have to configure anything else
- As a developer with several provider keys, I want an interactive prompt (provider + model) when running in a terminal, and a deterministic choice in CI, so that resolution is predictable everywhere
- As a team lead, I want to pin a provider/model in `specsync.json` (`aiProvider`, `aiModel`) so that all contributors generate specs consistently
- As a CI operator, I want to override the provider and model with environment variables (`SPECSYNC_AI_PROVIDER`, `SPECSYNC_AI_MODEL`, `SPECSYNC_AI_COMMAND`) so that I can configure generation in pipelines without editing project config
- As a developer using a custom or self-hosted gateway, I want an explicit trusted shell hatch (`aiCommand` / `SPECSYNC_AI_COMMAND`) or a custom `aiBaseUrl` so that I can route to any tool or OpenAI-compatible endpoint

## Acceptance Criteria

- Provider resolution follows `flag > env > config` (12-factor): `--provider` flag > `SPECSYNC_AI_COMMAND` env > `SPECSYNC_AI_PROVIDER` env > `aiCommand` config > `aiProvider` config > auto-detect
- Model precedence is the same shape: `--model` flag > `SPECSYNC_AI_MODEL` env > `aiModel` config > corvid-ai provider default
- All API providers (`anthropic`, `openai`, `openrouter`, `gemini`, `deepseek`, `groq`, `mistral`, `xai`, `together`, `ollama`) route through the shared `corvid-ai` crate (`corvid_ai::complete`) — there are no hand-rolled per-provider HTTP calls
- Auto-detect uses `<PROVIDER>_API_KEY` presence only, with no network probe: none set → keyless local Ollama; exactly one set → use it; multiple set → interactive prompt when stdin and stderr are TTYs, else the deterministic order `[Ollama, Anthropic, OpenAI, OpenRouter, Gemini, DeepSeek, Groq, Mistral, xAI, Together]`
- A set API key beats unkeyed local Ollama; auto-detect never selects a CLI shell-out provider
- The deprecated `claude` provider routes to the `anthropic` API (with a deprecation warning), never shelling out to `claude -p`
- Deprecated `copilot` and the no-pipe `cursor` providers still resolve via the legacy CLI path with a warning; `cursor` returns a clear "no CLI pipe mode" error with workarounds
- Ollama host resolves as `OLLAMA_HOST` env > `aiBaseUrl` config > `-cloud` routing (model tag contains `-cloud` and `OLLAMA_API_KEY` set ⇒ `https://ollama.com`) > `http://localhost:11434`; corvid-ai talks to it over the OpenAI-compatible `/v1` endpoint
- Missing API key for a key-required provider produces a clear error naming the expected env var and `aiApiKey`; the key check is skipped for Ollama and when a custom `aiBaseUrl` is set
- Source code input is capped at 30K chars per file and 150K chars total before prompting
- AI output is post-processed: code fences stripped, frontmatter delimiters validated, error if missing
- Generation times out after `aiTimeout` (default 120s); API timeout is carried into `corvid_ai::Settings`

## Constraints

- All HTTP transport is owned by `corvid-ai`; spec-sync carries only user overrides (model/key/base-url/timeout) in `corvid_ai::Settings`
- `corvid-ai` owns the endpoint registry, default models, `<PROVIDER>_API_KEY` resolution, and secret redaction in error strings
- `ResolvedProvider`'s `Debug` must redact the API key (`[REDACTED]`), since the derived `corvid_ai::Settings` debug would print it verbatim
- CLI shell-out (`aiCommand`) must write stdin on a background thread to avoid a stdout-pipe deadlock when the child streams tokens
- Must not panic on provider failure — every path returns `Result<_, String>` with an actionable message
- MSRV 1.89; resolution semantics and warning text are kept aligned with fledge

## Out of Scope

- Streaming token-by-token API output to the terminal (CLI path streams lines; API path does not)
- Caching AI responses across runs
- Fine-tuning or training custom models
- Providers requiring OAuth flows
- `AiProvider::default_model` / `default_base_url` — removed; corvid-ai owns these defaults

### REQ-ai-001

The system SHALL document deprecated AI compatibility paths according to their shipped major-version behavior.

Acceptance Criteria
- The Claude alias is described as deprecated but retained in 5.0.
- The trusted `aiCommand` escape hatch is described as deprecated but retained in 5.0.
