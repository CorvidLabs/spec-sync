---
title: "For AI Agents"
section: "Integrations"
order: 2
---

SpecSync is built for LLM-powered coding tools — structured output, machine-readable specs, and automated scaffolding.

---

## MCP Server Mode

SpecSync can run as an [MCP](https://modelcontextprotocol.io/) server, letting AI agents (Claude Code, Cursor, Windsurf, etc.) call SpecSync tools natively over stdio:

```bash
specsync mcp
```

This exposes tools: `specsync_check`, `specsync_generate`, `specsync_coverage`, `specsync_score`. Agents discover and invoke them via JSON-RPC — no CLI parsing needed.

Add to your agent's MCP config (e.g., `claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "specsync": {
      "command": "specsync",
      "args": ["mcp"]
    }
  }
}
```

---

## AI Providers (`--provider`)

AI calls go through the shared [`corvid-ai`](https://crates.io/crates/corvid-ai) crate — plain HTTP, no CLI tool required. Set `<PROVIDER>_API_KEY` and pick a provider:

```bash
specsync generate --provider anthropic        # uses ANTHROPIC_API_KEY
specsync generate --provider openai --model gpt-4o   # uses OPENAI_API_KEY
specsync generate --provider ollama           # keyless local Ollama (http://localhost:11434)
specsync generate                             # auto-detect (see resolution ladder below)
```

Supported providers: `anthropic`, `openai`, `openrouter`, `gemini`, `deepseek`, `groq`, `mistral`, `xai`, `together`, and `ollama` (local & keyless, or Ollama Cloud via `OLLAMA_API_KEY`).

**Deprecated:** `claude` now warns and routes to the `anthropic` API (no more `claude -p` shell-out); `copilot` and `cursor` are deprecated too.

`generate` enters AI mode when a provider is configured (`--provider`, `aiProvider`/`aiCommand` config, or any `SPECSYNC_AI_*` env var). With nothing configured it is template-only.

---

## Spec Quality Scoring

Score your specs on a 0–100 scale with actionable improvement suggestions:

```bash
specsync score                    # human-readable output
specsync score --json             # machine-readable scores
```

Scores are based on completeness, detail, API coverage, behavioral examples, and more. Use this in CI to enforce minimum spec quality.

---

## AI-Powered Generation (`--provider`)

`specsync generate --provider anthropic` reads your source code, sends it to an LLM, and generates source-aware specs. Purpose, Public API tables, Invariants, Error Cases — all derived from the code.

```bash
specsync generate --provider anthropic
#   Generating specs/auth/auth.spec.md with AI...
#     │ ---
#     │ module: auth
#     │ ...
#   ✓ Generated specs/auth/auth.spec.md (3 files)
```

### Choosing a provider and model

The provider is resolved `flag > env > config`:
1. `--provider <name>` flag
2. `SPECSYNC_AI_PROVIDER` environment variable
3. `aiProvider`/`ai_provider` in config (or `.specsync/config.local.toml` for per-developer overrides)

With nothing set, `generate` **auto-detects**:
- **No key anywhere → keyless local Ollama** (`http://localhost:11434`) — the zero-config default
- Exactly one `<PROVIDER>_API_KEY` set → that provider
- Several keys set → an interactive picker on a TTY, otherwise a deterministic order: Ollama, Anthropic, OpenAI, OpenRouter, Gemini, DeepSeek, Groq, Mistral, xAI, Together

The model is resolved the same way (`flag > env > config`):
1. `--model <id>` flag
2. `SPECSYNC_AI_MODEL` environment variable
3. `aiModel`/`ai_model` in config
4. Provider default — `anthropic` → `claude-sonnet-4-6`, Ollama → `llama3.3`. `openai` and `together` require an explicit model.

```toml
# .specsync/config.toml (or .specsync/config.local.toml for per-developer overrides)
ai_provider = "anthropic"
ai_model = "claude-sonnet-4-6"
ai_timeout = 120
```

```bash
export SPECSYNC_AI_PROVIDER=openrouter
export SPECSYNC_AI_MODEL=anthropic/claude-sonnet-4
specsync generate
```

If AI generation fails for a module, it falls back to template generation automatically.

### Shell escape hatch (`aiCommand`, deprecated)

`aiCommand`/`SPECSYNC_AI_COMMAND` remain as an explicit, **trusted shell escape hatch** — any command that reads a prompt on stdin and writes markdown to stdout. It is **deprecated** and **never auto-selected**; the HTTP providers above are preferred.

```toml
# .specsync/config.toml — deprecated trusted hatch
ai_command = "my-llm-wrapper --format markdown"
ai_timeout = 300
```

### Template mode (no `--provider`)

Without `--provider`, `specsync generate` scaffolds template specs with frontmatter populated and guided starter sections. Place `_template.spec.md` in your specs directory to control the generated structure.

---

## End-to-End Workflow

```bash
# One command: AI reads code, writes specs
specsync generate --provider anthropic

# Validate the generated specs against code
specsync check --json

# LLM fixes errors from JSON output, iterates until clean

# CI gate with full coverage
specsync check --strict --require-coverage 100
```

Each step produces machine-readable output. No human in the loop required (though humans can review at any step).

---

## Why SpecSync Works for LLMs

| Feature | Why it matters |
|---------|---------------|
| Plain markdown specs | Any LLM can read and write them — no custom format to learn |
| `--json` flag on every command | Structured output, no ANSI codes to strip |
| Exit code 0/1 | Pass/fail without parsing |
| Backtick-quoted names in API tables | Unambiguous extraction — first backtick-quoted string per row |
| `specsync generate` | Bootstrap from zero — LLM fills in content, not boilerplate |
| Deterministic validation | Same input → same output, no flaky checks |

---

## JSON Output Shapes

### `specsync check --json`

```json
{
  "passed": false,
  "errors": ["auth.spec.md: phantom export `oldFunction` not found in source"],
  "warnings": ["auth.spec.md: undocumented export `newHelper`"],
  "specs_checked": 12
}
```

- **Errors**: spec references something missing from code — must fix
- **Warnings**: code exports something the spec doesn't mention — informational
- **`--strict`**: promotes warnings to errors

### `specsync coverage --json`

```json
{
  "file_coverage": 85.33,
  "files_covered": 23,
  "files_total": 27,
  "loc_coverage": 79.12,
  "loc_covered": 4200,
  "loc_total": 5308,
  "modules": [{ "name": "helpers", "has_spec": false }],
  "uncovered_files": [{ "file": "src/helpers/utils.ts", "loc": 340 }]
}
```

Use `modules` with `has_spec: false` to identify what `generate` would scaffold. `uncovered_files` shows LOC per uncovered file, sorted by size — prioritize the largest gaps.

---

## Writing Specs Programmatically

1. Frontmatter requires `module`, `version`, `status`, `files`
2. Status values: `draft`, `review`, `stable`, `deprecated`
3. `files` must be non-empty, paths relative to project root
4. Public API tables: first backtick-quoted string per row is the export name
5. Default required sections: Purpose, Public API, Invariants, Behavioral Examples, Error Cases, Dependencies, Change Log

### Minimal valid spec

```markdown
---
module: mymodule
version: 1
status: draft
files:
  - src/mymodule.ts
---

# MyModule

## Purpose
Describe the module responsibility and caller-visible behavior.

## Public API

| Export | Description |
|--------|-------------|
| `myFunction` | Does something |

## Invariants
Document guarantees that must remain true.

## Behavioral Examples
Show the primary workflow with Given/When/Then examples.

## Error Cases
Document invalid input and failure handling.

## Dependencies
None

## Change Log

| Date | Change |
|------|--------|
| 2026-03-19 | Initial spec |
```

---

## Integration Patterns

| Pattern | Command | How |
|---------|---------|-----|
| **Pre-commit hook** | `specsync check --strict` | Block commits with spec errors |
| **PR review bot** | `specsync check --json` | Parse output, post as PR comment |
| **Bootstrap coverage** | `specsync generate --provider anthropic` | AI writes specs from source code |
| **Template scaffold** | `specsync generate` | Scaffold templates after adding new modules |
| **AI code review** | `specsync check --json` | Feed errors to LLM for spec updates |
| **Coverage gate** | `specsync check --strict --require-coverage 100` | CI enforces full coverage |
| **Quality gate** | `specsync score --json` | Enforce minimum spec quality scores |
| **MCP integration** | `specsync mcp` | Native tool access for AI agents |
