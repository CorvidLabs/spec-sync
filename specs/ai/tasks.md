---
spec: ai.spec.md
---

## Tasks

- [ ] Add streaming for API providers (the CLI path streams lines; API responses arrive whole)
- [ ] Add retry with backoff for transient API failures (rate limits, timeouts) — pending corvid-ai support
- [ ] Make the context-window cap (30K/file, 150K total) provider-aware instead of fixed char limits
- [ ] Plan removal of the remaining CLI providers (`copilot`, `cursor`) and the `aiCommand` hatch for spec-sync 5.0

## Done

- [x] Migrate all API HTTP calls to the shared `corvid-ai` crate (`corvid_ai::complete`); remove hand-rolled `call_anthropic_api` / `call_openai_api` / `call_gemini_api`
- [x] Collapse `ResolvedProvider` to `Cli(String)` | `Api(corvid_ai::Settings)` with key-redacting `Debug`
- [x] Add API providers `openrouter`, `gemini`, `deepseek`, `groq`, `mistral`, `xai`, `together`, and `ollama` (OpenAI-compatible HTTP)
- [x] Route the deprecated `claude` alias to the `anthropic` API with a warning (no more `claude -p` shell-out)
- [x] Default to keyless local Ollama (`http://localhost:11434`) when no provider key is configured
- [x] Implement the `flag > env > config` resolution ladder, adding `SPECSYNC_AI_PROVIDER` env
- [x] Implement key-presence auto-detection (no network probe): none→Ollama, one→use it, multiple→prompt or deterministic order
- [x] Add interactive provider + model prompt when multiple keys are set and stdin/stderr are TTYs
- [x] Implement Ollama host routing: `OLLAMA_HOST` > `aiBaseUrl` > `-cloud`+`OLLAMA_API_KEY` cloud routing > localhost, served over `/v1`
- [x] Add model precedence `--model` > `SPECSYNC_AI_MODEL` > `aiModel` > corvid-ai default, plus the `generate --model` flag
- [x] Remove `AiProvider::default_model` / `default_base_url` (corvid-ai owns defaults)
- [x] Source truncation (30K/file, 150K total) on UTF-8 char boundaries
- [x] Post-processing: code-fence stripping and frontmatter validation

## Gaps

- API completion paths are not exercised by live unit tests (would need a corvid-ai mock or recorded HTTP fixtures); coverage is via resolution/error-path tests plus integration fixtures that fail fast on missing keys
- The interactive multi-key prompt (`prompt_provider_and_model`) has no automated test — it requires a TTY

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
