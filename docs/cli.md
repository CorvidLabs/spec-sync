---
title: CLI Reference
layout: default
nav_order: 3
---

# CLI Reference
{: .no_toc }

<details open markdown="block">
  <summary>Table of contents</summary>
  {: .text-delta }
1. TOC
{:toc}
</details>

---

## Usage

```
specsync [command] [flags]
```

Default command is `check`.

---

## Commands

### `check`

Validate all specs against source code.

```bash
specsync check                          # basic validation
specsync check --strict                 # warnings become errors
specsync check --strict --require-coverage 100
specsync check --json                   # machine-readable output
```

Three validation stages:
1. **Structural** — required frontmatter fields, file existence, required sections
2. **API surface** — spec symbols vs. actual code exports
3. **Dependencies** — `depends_on` paths, `db_tables` against schema

### `coverage`

File and module coverage report.

```bash
specsync coverage
specsync coverage --json
```

### `generate`

Scaffold spec files for modules that don't have one. Uses `specs/_template.spec.md` if present.

```bash
specsync generate                       # template mode — stubs with TODOs
specsync generate --provider auto       # AI mode — auto-detect provider, writes real content
specsync generate --provider anthropic  # AI mode — use Anthropic API directly
```

With `--provider`, source code is piped to an LLM which generates filled-in specs (Purpose, Public API tables, Invariants, etc.). Use `--provider auto` to auto-detect an installed provider, or specify one by name (`anthropic`, `openai`, `command`). The `command` provider resolves from: `aiCommand` in config → `SPECSYNC_AI_COMMAND` env var → `claude -p --output-format text`. See [Configuration](configuration) for `aiCommand` and `aiTimeout`.

### `score`

Quality-score your spec files on a 0–100 scale with per-spec improvement suggestions.

```bash
specsync score                          # score all specs
specsync score --json                   # machine-readable scores
```

Scores are based on a weighted rubric: completeness, detail level, API table coverage, behavioral examples, and more.

### `mcp`

Start SpecSync as an MCP (Model Context Protocol) server over stdio. Enables AI agents like Claude Code, Cursor, and Windsurf to use SpecSync tools natively.

```bash
specsync mcp                            # start MCP server (stdio JSON-RPC)
```

Exposes tools: `specsync_check`, `specsync_generate`, `specsync_coverage`, `specsync_score`.

### `add-spec`

Scaffold a single spec with companion files (`requirements.md`, `tasks.md`, `context.md`).

```bash
specsync add-spec auth                     # creates specs/auth/auth.spec.md + companions
```

Companion files sit alongside the spec and give agents structured context:
- **`requirements.md`** — user stories, acceptance criteria, constraints (authored by Product/Design)
- **`tasks.md`** — outstanding work items for the module
- **`context.md`** — design decisions, constraints, history

### `init-registry`

Generate a `specsync-registry.toml` listing all modules in the project. Other projects reference your modules via this registry.

```bash
specsync init-registry                     # uses project folder name
specsync init-registry --name myapp        # custom registry name
```

Commit the generated file to your repo's default branch so `resolve --remote` can find it.

### `resolve`

Verify that all `depends_on` references in your specs actually exist. By default checks local paths only (no network).

```bash
specsync resolve                           # verify local refs
specsync resolve --remote                  # also verify cross-project refs via GitHub
```

Cross-project refs use the `owner/repo@module` syntax in `depends_on`. The `--remote` flag fetches the target repo's `specsync-registry.toml` from GitHub to confirm the module exists. See [Cross-Project References](cross-project-refs) for details.

### `hooks`

Install agent instruction files and git hooks so AI agents and contributors stay spec-aware.

```bash
specsync hooks install                     # install agent instructions + pre-commit hook
specsync hooks uninstall                   # remove installed hooks
specsync hooks status                      # check what's installed
```

Supports Claude Code (`CLAUDE.md`), Cursor (`.cursor/rules`), GitHub Copilot (`.github/copilot-instructions.md`), and pre-commit hooks.

### `compact`

Trim older changelog entries from specs to prevent unbounded growth.

```bash
specsync compact --keep 10              # keep last 10 entries per spec
specsync compact --keep 5 --dry-run     # preview what would be removed
```

### `archive-tasks`

Archive completed tasks from companion `tasks.md` files.

```bash
specsync archive-tasks                  # move completed tasks to archive section
specsync archive-tasks --dry-run        # preview what would be archived
```

### `view`

View specs filtered by role — shows only the sections relevant to a specific audience.

```bash
specsync view --role dev                # developer view
specsync view --role qa                 # QA view
specsync view --role product            # product manager view
specsync view --role agent              # AI agent view
specsync view --role dev --spec auth    # specific spec, developer view
```

### `diff`

Show API changes since a git ref.

```bash
specsync diff main                      # changes since main branch
specsync diff HEAD~5                    # changes in last 5 commits
specsync diff v1.0.0 --json            # machine-readable output
```

### `init`

Create a default `specsync.json` in the current directory.

```bash
specsync init
```

### `watch`

Live validation — re-runs on file changes with 500ms debounce. `Ctrl+C` to exit.

```bash
specsync watch
```

---

## Flags

| Flag | Description |
|:-----|:------------|
| `--strict` | Warnings become errors. Recommended for CI. |
| `--require-coverage N` | Fail if file coverage < N%. |
| `--root <path>` | Project root directory (default: cwd). |
| `--provider <name>` | Enable AI-powered generation and select provider: `auto` (auto-detect), `anthropic`, `openai`, or `command`. Without this flag, `generate` uses templates only. |
| `--format <fmt>` | Output format: `text` (default), `json`, or `markdown`. Markdown produces clean tables suitable for PRs and docs. |
| `--json` | Shorthand for `--format json`. Structured output, no color codes. |

---

## Exit Codes

| Code | Meaning |
|:-----|:--------|
| `0` | All checks passed |
| `1` | Errors found, warnings with `--strict`, or coverage below threshold |

---

## JSON Output

### Check

```json
{
  "passed": false,
  "errors": ["auth.spec.md: phantom export `oldFunction` not found in source"],
  "warnings": ["auth.spec.md: undocumented export `newHelper`"],
  "specs_checked": 12
}
```

### Coverage

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
