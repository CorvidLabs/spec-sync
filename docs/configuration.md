---
title: Configuration
layout: default
nav_order: 4
---

# Configuration
{: .no_toc }

SpecSync is configured via `specsync.json` in your project root. All fields are optional — sensible defaults apply.
{: .fs-6 .fw-300 }

<details open markdown="block">
  <summary>Table of contents</summary>
  {: .text-delta }
1. TOC
{:toc}
</details>

---

## Getting Started

```bash
specsync init
```

Creates `specsync.json` with defaults. SpecSync also works without a config file.

### TOML Config

SpecSync also supports `specsync.toml` as an alternative to JSON:

```toml
specs_dir = "specs"
source_dirs = ["src"]
schema_dir = "db/migrations"
ai_provider = "anthropic"
ai_model = "claude-sonnet-4-20250514"
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

Config resolution order: `specsync.json` → `specsync.toml` → defaults.

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
  "aiModel": "claude-sonnet-4-20250514",
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
| `aiProvider` | `string?` | — | AI provider name: `claude`, `anthropic`, `openai`, `ollama`, `copilot`, or `custom` |
| `aiModel` | `string?` | Provider default | Model name override (e.g., `"claude-sonnet-4-20250514"`, `"gpt-4o"`, `"mistral"`) |
| `aiCommand` | `string?` | — | Custom CLI command for AI generation (reads stdin prompt, writes stdout markdown) |
| `aiApiKey` | `string?` | — | API key for `anthropic` or `openai` providers (prefer env vars `ANTHROPIC_API_KEY` / `OPENAI_API_KEY` instead) |
| `aiBaseUrl` | `string?` | — | Custom base URL for API providers (e.g., for proxies or self-hosted endpoints) |
| `aiTimeout` | `number?` | `120` | Seconds before AI command times out per module |
| `exportLevel` | `string?` | `"member"` | Export validation depth: `"type"` (classes/structs only) or `"member"` (all public symbols) |
| `modules` | `object?` | `{}` | Custom module definitions mapping module names to `{ files, depends_on }` |
| `rules` | `object?` | `{}` | Custom validation rules (see [Validation Rules](#validation-rules) below) |
| `taskArchiveDays` | `number?` | — | Days after which completed tasks in companion `tasks.md` files are auto-archived |
| `github` | `object?` | — | GitHub integration settings (see [GitHub Config](#github-config) below) |

---

## AI Provider Resolution

When you run `specsync generate --provider auto`, the provider is resolved in this order:

1. `--provider` CLI flag (explicit)
2. `aiCommand` in config (custom command always wins)
3. `aiProvider` in config (resolved to CLI or API)
4. `SPECSYNC_AI_COMMAND` env var
5. Auto-detect: installed CLIs first (`claude`, `ollama`, `copilot`), then API keys (`ANTHROPIC_API_KEY`, `OPENAI_API_KEY`)

### API Providers

The `anthropic` and `openai` providers call their respective APIs directly — no CLI tool needed. Just set the API key:

```json
{
  "aiProvider": "anthropic"
}
```

Then set `ANTHROPIC_API_KEY` in your environment (or use `aiApiKey` in config for local use — **not recommended for shared repos**).

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
