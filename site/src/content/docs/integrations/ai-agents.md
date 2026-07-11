---
title: "For Coding Agents"
section: "Integrations"
order: 2
---

SpecSync 5.0 is agent-native without embedding an inference client. The core is deterministic: it scaffolds markdown, validates contracts, records approvals and evidence, and never stores model credentials or sends source to a provider.

## Trust Boundary

SpecSync does not accept provider/model flags, API keys, model endpoints, or AI commands. `SPECSYNC_AI_COMMAND` is never executed. Your existing coding agent may read and refine specs using its own configured credentials and permissions; SpecSync remains the local workflow and validation engine.

```text
coding agent credentials + permissions
             │
             ▼
read code → refine markdown
             │
             ▼
SpecSync deterministic check + SDD evidence
```

## Native Verified-SDD Skills

```bash
specsync agents install
```

Claude Code, Cursor, Codex, and Gemini receive a native project skill. Claude, Cursor, and Gemini also receive create-spec and create-change commands. The lifecycle skill requires agents to:

- create meaningful work through `specsync change new --json`;
- present deterministic interview questions to the user;
- never invent or self-grant either human approval;
- implement only after definition approval;
- show fresh verification evidence before closing approval;
- apply semantic deltas and archive through deterministic CLI operations.

Changing agents does not change lifecycle state or artifact semantics because the CLI remains the shared workflow engine.

## MCP Server

```bash
specsync mcp
```

The stdio MCP server exposes deterministic `specsync_check`, `specsync_generate`, `specsync_coverage`, and `specsync_score` tools. `specsync_generate` creates local templates only. Legacy AI/provider/model/credential/endpoint/command arguments are rejected rather than silently implying inference occurred.

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

## End-to-End Agent Workflow

```bash
specsync generate                              # deterministic local scaffolds
specsync agents install                        # native agent instructions
specsync check --json                          # structured feedback
# coding agent refines markdown from feedback
specsync check --strict --require-coverage 100 # deterministic release gate
```

For contract-changing delivery, start with `specsync change new`, complete the selected artifacts and semantic deltas, obtain definition approval, then verify and obtain closing approval. Agents and humans use the same state machine.

## Why It Works for Agents

| Feature | Why it matters |
|:--------|:---------------|
| Plain markdown specs | Any coding agent can read and edit them |
| `--json` output | Structured feedback without terminal parsing |
| Exit code 0/1 | Deterministic pass/fail |
| Backtick-quoted API names | Unambiguous export matching |
| Deterministic scaffolding | Reproducible output with no model dependency |
| Human approval gates | Agents cannot self-authorize contract changes |
| MCP + native skills | Integration without moving credentials into SpecSync |

## JSON Output

```json
{
  "passed": false,
  "errors": ["auth.spec.md: phantom export `oldFunction` not found in source"],
  "warnings": ["auth.spec.md: undocumented export `newHelper`"],
  "specs_checked": 12
}
```

Errors identify stale contracts; warnings identify undocumented code. `--strict` promotes warnings to failures.

## Writing Specs Programmatically

1. Frontmatter requires `module`, `version`, `status`, and `files`.
2. Public API table names are backtick-quoted.
3. Requirements use stable module-scoped IDs and normative SHALL statements.
4. Companion `tasks.md`, `requirements.md`, `context.md`, and `testing.md` preserve work and evidence context.
5. Always finish with `specsync check --strict` and `specsync score`.

## Integration Patterns

| Pattern | Command | Purpose |
|:--------|:--------|:--------|
| Bootstrap coverage | `specsync generate` | Deterministic templates for uncovered modules |
| Native agent workflow | `specsync agents install` | Project skills and supported slash commands |
| MCP integration | `specsync mcp` | Structured local tool access |
| PR review | `specsync check --json` | Feed deterministic drift to a coding agent |
| Coverage gate | `specsync check --strict --require-coverage 100` | Enforce complete release coverage |
| Quality gate | `specsync score --json` | Improve low-quality contracts |
