---
title: For AI Agents
layout: default
nav_order: 6
---

# For AI Agents
{: .no_toc }

SpecSync is built for LLM-powered coding tools — structured output, machine-readable specs, and automated scaffolding.
{: .fs-6 .fw-300 }

<details open markdown="block">
  <summary>Table of contents</summary>
  {: .text-delta }
1. TOC
{:toc}
</details>

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

## Direct API Providers (`--provider`)

Instead of shelling out to a CLI for AI generation, SpecSync can call Anthropic or OpenAI APIs directly:

```bash
specsync generate --ai --provider anthropic   # uses ANTHROPIC_API_KEY
specsync generate --ai --provider openai      # uses OPENAI_API_KEY
specsync generate --ai --provider command     # default: shells out to aiCommand
```

This removes the dependency on having Claude CLI or other tools installed — just set the API key.

---

## Spec Quality Scoring

Score your specs on a 0–100 scale with actionable improvement suggestions:

```bash
specsync score                    # human-readable output
specsync score --json             # machine-readable scores
```

Scores are based on completeness, detail, API coverage, behavioral examples, and more. Use this in CI to enforce minimum spec quality.

---

## AI-Powered Generation (`--ai`)

`specsync generate --ai` reads your source code, sends it to an LLM, and generates specs with real content — not just templates with TODOs. Purpose, Public API tables, Invariants, Error Cases — all filled in from the code.

```bash
specsync generate --ai
#   Generating specs/auth/auth.spec.md with AI...
#     │ ---
#     │ module: auth
#     │ ...
#   ✓ Generated specs/auth/auth.spec.md (3 files)
```

### Configuring the AI command

The AI command is resolved in order:
1. `"aiCommand"` in `specsync.json`
2. `SPECSYNC_AI_COMMAND` environment variable
3. `claude -p --output-format text` (default, requires Claude CLI)

Any command that reads a prompt from stdin and writes markdown to stdout works:

```json
{
  "aiCommand": "claude -p --output-format text",
  "aiTimeout": 300
}
```

```json
{
  "aiCommand": "ollama run llama3",
  "aiTimeout": 60
}
```

If AI generation fails for a module, it falls back to template generation automatically.

### Template mode (no `--ai`)

Without `--ai`, `specsync generate` scaffolds template specs — frontmatter populated, required sections stubbed with TODOs. Place `_template.spec.md` in your specs directory to control the generated structure.

---

## End-to-End Workflow

```bash
# One command: AI reads code, writes specs
specsync generate --ai

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
TODO

## Public API

| Export | Description |
|--------|-------------|
| `myFunction` | Does something |

## Invariants
TODO

## Behavioral Examples
TODO

## Error Cases
TODO

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
| **Bootstrap coverage** | `specsync generate --ai` | AI writes specs from source code |
| **Template scaffold** | `specsync generate` | Scaffold templates after adding new modules |
| **AI code review** | `specsync check --json` | Feed errors to LLM for spec updates |
| **Coverage gate** | `specsync check --strict --require-coverage 100` | CI enforces full coverage |
| **Quality gate** | `specsync score --json` | Enforce minimum spec quality scores |
| **MCP integration** | `specsync mcp` | Native tool access for AI agents |
