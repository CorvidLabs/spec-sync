---
spec: ai.spec.md
---

## User Stories

- As a developer, I want spec-sync to auto-detect which AI provider is available on my system so that I don't need to manually configure it
- As a team lead, I want to specify an AI provider in config so that all team members use the same provider for consistent spec generation
- As a developer using Claude, I want to generate specs via the CLI binary so that I don't need an API key
- As a developer with an Anthropic API key, I want spec-sync to call the API directly so that I get faster, more reliable generation without needing the CLI installed
- As a developer using Ollama, I want local AI generation so that my code never leaves my machine
- As a CI operator, I want to set the AI provider via environment variable (SPECSYNC_AI_COMMAND) so that I can configure generation in pipelines without modifying project config

## Acceptance Criteria

- [ ] CLI flag `--provider` overrides all other provider settings
- [ ] Config field `aiCommand` overrides `aiProvider` when both are set
- [ ] Auto-detection tries CLI providers (claude, ollama, copilot) before API providers (anthropic, openai)
- [ ] Cursor provider returns a clear error explaining it cannot be used (no stdin/stdout pipe mode)
- [ ] Source code input is capped at 150KB total and 30KB per individual file
- [ ] AI response has code fences stripped and frontmatter validated before returning
- [ ] Generation times out after configurable `aiTimeout` (default 120s)
- [ ] API providers (Anthropic, OpenAI) use direct HTTP calls, not CLI binaries
- [ ] Missing API key for API providers produces a clear error message naming the expected env var
- [ ] Provider resolution falls back gracefully: CLI flag > aiCommand > aiProvider > env var > auto-detect

## Constraints

- No external crate dependencies for HTTP beyond ureq (already in use)
- AI prompt must include all relevant source files to produce accurate specs
- Timeout must be enforced even if the AI provider hangs (channel-based async)
- Must not panic on provider failure — return Result with actionable error message

## Out of Scope

- Streaming token-by-token output to the terminal
- Caching AI responses across runs
- Fine-tuning or training custom models
- Supporting providers that require OAuth flows
