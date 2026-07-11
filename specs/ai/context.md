---
spec: ai.spec.md
---

## Key Decisions

- **corvid-ai owns all HTTP**: Every API provider routes through the shared `corvid-ai` crate via `corvid_ai::complete(&settings, &completion)`. spec-sync no longer hand-rolls Anthropic / OpenAI / Gemini wire shapes; corvid-ai owns the endpoint registry, default models, `<PROVIDER>_API_KEY` resolution, and secret redaction. This keeps spec-sync aligned with fledge.
- **`ResolvedProvider` has two variants**: `Cli(String)` (legacy shell-out) and `Api(corvid_ai::Settings)`. Its `Debug` impl is hand-written to redact the API key as `[REDACTED]` — the derived `Settings` debug would leak it.
- **`flag > env > config` resolution (12-factor)**: `--provider` > `SPECSYNC_AI_COMMAND` env > `SPECSYNC_AI_PROVIDER` env > `aiCommand` config > `aiProvider` config > auto-detect. Within env and config tiers, the command hatch outranks the provider name. Model precedence mirrors this: `--model` > `SPECSYNC_AI_MODEL` > `aiModel` > corvid-ai default.
- **Key-presence auto-detect, no network probe**: none set → keyless local Ollama; exactly one → use it; multiple → interactive prompt (provider + model) when stdin & stderr are TTYs, else the deterministic order `[Ollama, Anthropic, OpenAI, OpenRouter, Gemini, DeepSeek, Groq, Mistral, xAI, Together]`. A set key beats unkeyed local Ollama, and auto-detect never shells out to a CLI.
- **`claude` → anthropic API**: The deprecated `claude` provider resolves to the `anthropic` API (with a warning); it no longer shells out to `claude -p` and remains available for 5.0 compatibility.
- **Ollama host routing**: `OLLAMA_HOST` env > `aiBaseUrl` config > `-cloud` model tag + `OLLAMA_API_KEY` ⇒ `https://ollama.com` > `http://localhost:11434`. corvid-ai speaks to it over the OpenAI-compatible `/v1` endpoint; default model `llama3.3`. The eager key check is skipped for Ollama and whenever `aiBaseUrl` is set (self-hosted/proxy gateways).
- **Source truncation, not token counting**: 30K chars/file, 150K total, truncated on UTF-8 char boundaries — avoids a tokenizer dependency while staying inside context windows.
- **Post-processing guards disk writes**: AI output has code fences stripped and frontmatter delimiters validated before it can be written as a spec.

## Files to Read First

- `src/ai.rs` — provider resolution (`resolve_ai_provider`), the `ResolvedProvider` enum, prompt building, `corvid-ai` dispatch (`run_provider`), and post-processing all live here.
- `src/types.rs` — `AiProvider` (with `is_api_provider`, `api_key_env_var`, `detection_order`, `from_str_loose`) and `SpecSyncConfig`.
- `src/commands/generate.rs` — wires `--provider`/`--model` and the env/config precedence into `resolve_ai_provider`.
- The `corvid-ai` crate — `Settings`, `Completion`, `complete`, `DEFAULT_PROVIDER`; owns endpoints, default models, and key resolution.

## Current Status

Fully implemented and stable. API providers (`anthropic`, `openai`, `openrouter`, `gemini`, `deepseek`, `groq`, `mistral`, `xai`, `together`, `ollama`) all go through corvid-ai. Deprecated paths: `claude` (→ anthropic API), `copilot`, `cursor` (legacy CLI), plus the explicit `aiCommand` shell hatch. Auto-detect defaults to keyless local Ollama. MSRV 1.89. Test configuration construction is kept warning-free against the current stable Clippy gate used by CI.

## Notes

- `resolve_ai_command` remains as a thin compatibility alias over `resolve_ai_provider` for older tests; new code uses `resolve_ai_provider`.
- The CLI shell-out path writes stdin on a background thread to avoid a stdout-pipe deadlock when a child streams tokens, and shows a spinner / live `│`-prefixed lines on stderr.
- Eager key checks are UX-only "fail fast" — corvid-ai validates again at request time.
