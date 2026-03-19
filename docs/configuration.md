---
title: Configuration
layout: default
nav_order: 4
---

# Configuration
{: .no_toc }

SpecSync is configured via a `specsync.json` file in your project root.
{: .fs-6 .fw-300 }

<details open markdown="block">
  <summary>Table of contents</summary>
  {: .text-delta }
1. TOC
{:toc}
</details>

---

## Getting Started

Generate a default config:

```bash
specsync init
```

This creates `specsync.json` with sensible defaults. SpecSync works without a config file too — it uses the defaults listed below.

---

## Full Config

```json
{
  "specsDir": "specs",
  "sourceDirs": ["src"],
  "schemaDir": "db/migrations",
  "schemaPattern": "CREATE (?:VIRTUAL )?TABLE(?:\\s+IF NOT EXISTS)?\\s+(\\w+)",
  "requiredSections": [
    "Purpose",
    "Public API",
    "Invariants",
    "Behavioral Examples",
    "Error Cases",
    "Dependencies",
    "Change Log"
  ],
  "excludeDirs": ["__tests__"],
  "excludePatterns": [
    "**/__tests__/**",
    "**/*.test.ts",
    "**/*.spec.ts"
  ],
  "sourceExtensions": []
}
```

---

## Options

### `specsDir`

| Type | Default |
|:-----|:--------|
| `string` | `"specs"` |

Directory containing your `*.spec.md` files. Searched recursively.

### `sourceDirs`

| Type | Default |
|:-----|:--------|
| `string[]` | `["src"]` |

Source directories to scan for exports and coverage. SpecSync looks for source files referenced in spec frontmatter relative to the project root, but uses `sourceDirs` to determine overall coverage.

### `schemaDir`

| Type | Default |
|:-----|:--------|
| `string?` | — |

Directory containing SQL schema files. When set, SpecSync validates that `db_tables` listed in spec frontmatter actually exist as `CREATE TABLE` statements in your schema files.

### `schemaPattern`

| Type | Default |
|:-----|:--------|
| `string?` | `CREATE (?:VIRTUAL )?TABLE(?:\s+IF NOT EXISTS)?\s+(\w+)` |

Custom regex to extract table names from schema files. The first capture group should match the table name. Override this if your schema uses non-standard syntax.

### `requiredSections`

| Type | Default |
|:-----|:--------|
| `string[]` | `["Purpose", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"]` |

Markdown sections that every spec file must include. Matched against `## Heading` lines in the spec body.

### `excludeDirs`

| Type | Default |
|:-----|:--------|
| `string[]` | `["__tests__"]` |

Directory names to skip entirely when scanning for source files during coverage computation.

### `excludePatterns`

| Type | Default |
|:-----|:--------|
| `string[]` | `["**/__tests__/**", "**/*.test.ts", "**/*.spec.ts"]` |

Glob patterns for files to exclude from coverage scanning. These are in addition to the language-specific test file exclusions that SpecSync applies automatically.

### `sourceExtensions`

| Type | Default |
|:-----|:--------|
| `string[]` | All supported extensions |

Restrict analysis to specific file extensions. When empty (the default), SpecSync considers all supported language extensions. Set this to focus on specific languages:

```json
{
  "sourceExtensions": ["ts", "tsx"]
}
```

---

## Example Configs

### TypeScript project

```json
{
  "specsDir": "specs",
  "sourceDirs": ["src"],
  "excludePatterns": [
    "**/__tests__/**",
    "**/*.test.ts",
    "**/*.spec.ts",
    "**/*.d.ts"
  ]
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

### Minimal (just the essentials)

```json
{
  "requiredSections": ["Purpose", "Public API"]
}
```
