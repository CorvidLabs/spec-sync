---
spec: ai.spec.md
---

## Tasks

- [ ] Add streaming support for API providers (Anthropic and OpenAI) to show incremental output
- [ ] Support configurable context window size per provider instead of hardcoded 150KB cap
- [ ] Add retry logic with backoff for transient API failures (rate limits, timeouts)
- [ ] Support provider-specific model selection (e.g., `aiModel: "claude-sonnet-4-20250514"`)

## Done

- [x] Implement 5-stage provider resolution chain
- [x] Add CLI providers: Claude, Ollama, GitHub Copilot
- [x] Add API providers: Anthropic Messages API, OpenAI Chat Completions
- [x] Add Custom command provider
- [x] Auto-detect available providers
- [x] Source truncation (30KB/file, 150KB total)
- [x] Post-processing: code fence stripping, frontmatter validation

## Gaps

- No unit tests for API call paths (would need mocking or integration test infrastructure)
- Custom command provider has no validation that the command exists before running

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
