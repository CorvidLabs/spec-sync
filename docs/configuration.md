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
ai_command = "claude -p --output-format text"
ai_timeout = 120
required_sections = ["Purpose", "Requirements", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"]
exclude_dirs = ["__tests__"]
exclude_patterns = ["**/__tests__/**", "**/*.test.ts"]
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
  "requiredSections": ["Purpose", "Requirements", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"],
  "excludeDirs": ["__tests__"],
  "excludePatterns": ["**/__tests__/**", "**/*.test.ts", "**/*.spec.ts"],
  "sourceExtensions": [],
  "aiCommand": "claude -p --output-format text",
  "aiTimeout": 120
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
| `requiredSections` | `string[]` | 8 defaults | Markdown `##` sections every spec must include |
| `excludeDirs` | `string[]` | `["__tests__"]` | Directory names skipped during coverage scanning |
| `excludePatterns` | `string[]` | Common test globs | File patterns excluded from coverage (additive with language-specific test exclusions) |
| `sourceExtensions` | `string[]` | All supported | Restrict to specific extensions (e.g., `["ts", "rs"]`) |
| `aiCommand` | `string?` | `claude -p ...` | Command for `generate --provider command` (reads stdin prompt, writes stdout markdown) |
| `aiTimeout` | `number?` | `120` | Seconds before AI command times out per module |

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
