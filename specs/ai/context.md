---
spec: ai.spec.md
---

## Key Decisions

- **5-level provider resolution**: CLI flag > config `aiCommand` > config `aiProvider` > env var `SPECSYNC_AI_COMMAND` > auto-detect. This ensures explicit choices always win over implicit detection.
- **CLI vs API separation**: CLI providers (Claude, Ollama, Copilot) shell out to installed tools; API providers (Anthropic, OpenAI) make direct HTTP calls via ureq. This avoids requiring any CLI tool to be installed if you have an API key.
- **Cursor explicitly errors**: The Cursor IDE doesn't expose a CLI pipe, so selecting it produces a clear error directing users to use Claude or Copilot instead.
- **Source truncation**: Files capped at 30KB each, 150KB total, to stay within provider context limits without requiring token counting.
- **Post-processing**: AI output is stripped of code fences and validated for frontmatter before being written, preventing malformed specs from reaching disk.

## Files to Read First

- `src/ai.rs` — Single-file module; provider resolution, prompt building, API calls, and post-processing all live here.

## Current Status

Fully implemented. All 6 providers work (Claude CLI, Ollama, Copilot, Anthropic API, OpenAI API, Custom command). Auto-detection probes CLI tools first, then checks for API keys.

## Notes

- The `ureq` crate is the only HTTP dependency in the project — used here and in the registry module for remote fetches.
- Spinner and live-line output during AI generation provides user feedback without cluttering the terminal.
