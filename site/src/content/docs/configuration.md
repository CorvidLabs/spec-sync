---
title: "Configuration"
section: "Reference"
order: 1
---

Canonical validation is configured through `.specsync/config.toml`. The opt-in 5.0 SDD lifecycle uses the versioned `.specsync/sdd.json` policy so upgrading an existing project cannot silently enable new CI gates.

---

## Getting Started

```bash
specsync init
```

Creates `.specsync/config.toml` (v4) with defaults. SpecSync also works without a config file.

### TOML Config

Config resolution order: `.specsync/config.toml` → `.specsync/config.json` → `.specsync.toml` (legacy) → `specsync.json` (legacy) → defaults. If `.specsync/config.local.toml` exists (gitignored), it's merged on top for per-developer overrides.

Example:

```toml
specs_dir = "specs"
source_dirs = ["src"]
schema_dir = "db/migrations"
ai_provider = "anthropic"
ai_model = "claude-sonnet-4-6"
ai_timeout = 120
export_level = "member"
required_sections = ["Purpose", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"]
exclude_dirs = ["__tests__"]
exclude_patterns = ["**/__tests__/**", "**/*.test.ts"]
task_archive_days = 30

[rules]
max_changelog_entries = 20
require_behavioral_examples = true
min_invariants = 1

[github]
drift_labels = ["spec-drift"]
verify_issues = true
```

Config resolution order: `.specsync/config.toml` → `.specsync/config.json` → `.specsync.toml` (legacy) → `specsync.json` (legacy) → defaults. Per-developer overrides via `.specsync/config.local.toml` are merged on top.

### SDD Policy

New projects receive `.specsync/sdd.json`; existing projects create it with `specsync change adopt`:

```json
{
  "version": 1,
  "enabled": true,
  "require_change_for_meaningful_files": true,
  "meaningful_paths": ["src/", "tests/", "site/", ".github/", "Cargo.toml"],
  "ignored_paths": [".specsync/", "specs/"],
  "verification_commands": ["fledge run test"],
  "custom_artifacts": {},
  "principles_file": null
}
```

Verification commands are explicit argument lists executed without a shell. Shell operators, substitutions, pipes, and redirections are rejected. `custom_artifacts` maps an artifact name to a project-owned Markdown template path; `principles_file` optionally supplies project governance to interviews and agents.

---

## Full Config

```json
{
  "specsDir": "specs",
  "sourceDirs": ["src"],
  "schemaDir": "db/migrations",
  "schemaPattern": "CREATE (?:VIRTUAL )?TABLE(?:\\s+IF NOT EXISTS)?\\s+(\\w+)",
  "requiredSections": ["Purpose", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"],
  "excludeDirs": ["__tests__"],
  "excludePatterns": ["**/__tests__/**", "**/*.test.ts", "**/*.spec.ts"],
  "sourceExtensions": [],
  "exportLevel": "member",
  "aiProvider": "anthropic",
  "aiModel": "claude-sonnet-4-6",
  "aiCommand": null,
  "aiApiKey": null,
  "aiBaseUrl": null,
  "aiTimeout": 120,
  "taskArchiveDays": 30,
  "modules": {},
  "rules": {
    "maxChangelogEntries": 20,
    "requireBehavioralExamples": true,
    "minInvariants": 1,
    "maxSpecSizeKb": 50,
    "requireDependsOn": false
  },
  "github": {
    "repo": "owner/repo",
    "driftLabels": ["spec-drift"],
    "verifyIssues": true
  }
}
```

---

## Options

| Option | Type | Default | Description |
|:-------|:-----|:--------|:------------|
| `specsDir` | `string` | `"specs"` | Directory containing `*.spec.md` files (searched recursively) |
| `sourceDirs` | `string[]` | `["src"]` | Source directories for coverage analysis |
| `schemaDir` | `string?` | — | SQL schema directory for `db_tables` validation |
| `schemaPattern` | `string?` | `CREATE TABLE` regex | Custom regex for extracting table names (first capture group = table name) |
| `requiredSections` | `string[]` | 7 defaults | Markdown `##` sections every spec must include |
| `excludeDirs` | `string[]` | `["__tests__"]` | Directory names skipped during coverage scanning |
| `excludePatterns` | `string[]` | Common test globs | File patterns excluded from coverage (additive with language-specific test exclusions) |
| `sourceExtensions` | `string[]` | All supported | Restrict to specific extensions (e.g., `["ts", "rs"]`) |
| `aiProvider` | `string?` | — | AI provider: `anthropic`, `openai`, `openrouter`, `gemini`, `deepseek`, `groq`, `mistral`, `xai`, `together`, or `ollama`. Deprecated: `claude` (routes to `anthropic`), `copilot`, `cursor`. Overridable via `--provider` / `SPECSYNC_AI_PROVIDER` |
| `aiModel` | `string?` | Provider default | Model name override (e.g., `"claude-sonnet-4-6"`, `"gpt-4o"`, `"llama3.3"`). Overridable via `--model` / `SPECSYNC_AI_MODEL`. `openai` and `together` require an explicit model |
| `aiCommand` | `string?` | — | **Deprecated** trusted shell escape hatch — a command that reads a prompt on stdin and writes markdown to stdout. Never auto-selected; prefer the HTTP providers above. Overridable via `SPECSYNC_AI_COMMAND` |
| `aiApiKey` | `string?` | — | API key for the selected provider (prefer the per-provider env var `<PROVIDER>_API_KEY`, e.g. `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `OLLAMA_API_KEY`) |
| `aiBaseUrl` | `string?` | — | Custom base URL for API providers (e.g., proxies, self-hosted endpoints, or a non-default Ollama host via `OLLAMA_HOST`) |
| `aiTimeout` | `number?` | `120` | Seconds before AI command times out per module |
| `exportLevel` | `string?` | `"member"` | Export validation depth: `"type"` (classes/structs only) or `"member"` (all public symbols) |
| `modules` | `object?` | `{}` | Custom module definitions mapping module names to `{ files, depends_on }` |
| `rules` | `object?` | `{}` | Custom validation rules (see [Validation Rules](#validation-rules) below) |
| `taskArchiveDays` | `number?` | — | Days after which completed tasks in companion `tasks.md` files are auto-archived |
| `github` | `object?` | — | GitHub integration settings (see [GitHub Config](#github-config) below) |

---

## AI Provider Resolution

AI calls go through the shared [`corvid-ai`](https://crates.io/crates/corvid-ai) crate over plain HTTP — no CLI tool required. The **provider** is resolved `flag > env > config`:

1. `--provider` CLI flag
2. `SPECSYNC_AI_PROVIDER` env var
3. `aiProvider`/`ai_provider` in config — shared `config.toml` first, then `.specsync/config.local.toml` (gitignored, per-developer overrides)

With nothing set, `generate` auto-detects:
- **No `<PROVIDER>_API_KEY` anywhere → keyless local Ollama** (`http://localhost:11434`) — the zero-config default
- Exactly one `<PROVIDER>_API_KEY` set → that provider
- Several keys set → an interactive picker on a TTY, otherwise a deterministic order: Ollama, Anthropic, OpenAI, OpenRouter, Gemini, DeepSeek, Groq, Mistral, xAI, Together

The **model** follows the same `flag > env > config` precedence: `--model` > `SPECSYNC_AI_MODEL` > `aiModel`/`ai_model` config > provider default (`anthropic` → `claude-sonnet-4-6`, Ollama → `llama3.3`; `openai` and `together` require an explicit model).

`aiTimeout` (default `120`s) controls the per-module AI timeout. `aiBaseUrl` (or `OLLAMA_HOST` for Ollama) points a provider at a custom endpoint.

> **Multi-agent teams**: Don't put `ai_provider` or `ai_command` in the shared `config.toml`. Instead, each contributor creates `.specsync/config.local.toml` with their preferred AI settings. This file is automatically gitignored.

### API Providers

Every provider calls its HTTP API directly — no CLI tool needed. Just set the provider and its key:

```json
{
  "aiProvider": "anthropic"
}
```

Then set `ANTHROPIC_API_KEY` (or the relevant `<PROVIDER>_API_KEY`) in your environment — or use `aiApiKey` in config for local use (**not recommended for shared repos**). Local Ollama needs no key at all.

### Shell escape hatch (`aiCommand`)

`aiCommand`/`SPECSYNC_AI_COMMAND` remain as an explicit, **trusted shell escape hatch** — a command that reads a prompt on stdin and writes markdown to stdout. It is **deprecated** and **never auto-selected**; prefer the HTTP providers above.

---

## Validation Rules

Fine-tune validation behavior with the `rules` object:

```json
{
  "rules": {
    "maxChangelogEntries": 20,
    "requireBehavioralExamples": true,
    "minInvariants": 2,
    "maxSpecSizeKb": 50,
    "requireDependsOn": false
  }
}
```

| Rule | Type | Description |
|:-----|:-----|:------------|
| `maxChangelogEntries` | `number?` | Warn if a spec's Change Log exceeds this many entries |
| `requireBehavioralExamples` | `bool?` | Require at least one Behavioral Example scenario |
| `minInvariants` | `number?` | Minimum number of invariants required per spec |
| `maxSpecSizeKb` | `number?` | Warn if spec file exceeds this size in KB |
| `requireDependsOn` | `bool?` | Require non-empty `depends_on` in frontmatter |

---

## GitHub Config

Configure GitHub integration for drift detection and issue verification:

```json
{
  "github": {
    "repo": "owner/repo",
    "driftLabels": ["spec-drift"],
    "verifyIssues": true
  }
}
```

| Option | Type | Default | Description |
|:-------|:-----|:--------|:------------|
| `repo` | `string?` | Auto-detected | Repository in `owner/repo` format (auto-detected from git remote) |
| `driftLabels` | `string[]` | `["spec-drift"]` | Labels applied when creating drift issues |
| `verifyIssues` | `bool` | `true` | Whether to verify linked issues exist during `specsync check` |

---

## Custom Module Definitions

Map custom module names to specific files when auto-detection doesn't fit your layout:

```json
{
  "modules": {
    "auth": {
      "files": ["src/auth/service.ts", "src/auth/middleware.ts"],
      "dependsOn": ["database"]
    },
    "api": {
      "files": ["src/routes/"],
      "dependsOn": ["auth", "database"]
    }
  }
}
```

Module definitions override the default subdirectory/flat-file discovery for `specsync generate` and `specsync coverage`.

---

## Example Configs

### TypeScript project

```json
{
  "specsDir": "specs",
  "sourceDirs": ["src"],
  "excludePatterns": ["**/__tests__/**", "**/*.test.ts", "**/*.spec.ts", "**/*.d.ts"]
}
```

### Rust project

```json
{
  "specsDir": "specs",
  "sourceDirs": ["src"],
  "sourceExtensions": ["rs"]
}
```

### Monorepo

```json
{
  "specsDir": "docs/specs",
  "sourceDirs": ["packages/core/src", "packages/api/src"],
  "schemaDir": "packages/db/migrations"
}
```

### Minimal

```json
{
  "requiredSections": ["Purpose", "Public API"]
}
```
