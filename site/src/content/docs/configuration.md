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

Config resolution order: `.specsync/config.toml` → `.specsync/config.json` → `.specsync.toml` (legacy) → `specsync.json` (legacy) → defaults.

Example:

```toml
specs_dir = "specs"
source_dirs = ["src"]
schema_dir = "db/migrations"
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

SpecSync 5.0 does not load provider credentials or local AI overrides. Generation is deterministic and local.

### SDD Policy

New projects receive `.specsync/sdd.json`; existing projects create it with `specsync change adopt`:

```json
{
  "version": 1,
  "enabled": true,
  "require_change_for_meaningful_files": true,
  "meaningful_paths": ["src/", "tests/", "site/", ".github/", "Cargo.toml", ".specsync/sdd.json", ".specsync/config.toml"],
  "ignored_paths": [".specsync/", "specs/"],
  "verification_commands": ["fledge run test"],
  "custom_artifacts": {},
  "principles_file": null
}
```

Verification commands are explicit argument lists executed without a shell. Shell operators, substitutions, pipes, and redirections are rejected. `custom_artifacts` maps an artifact name to a project-owned Markdown template path; `principles_file` optionally supplies project governance to interviews and agents.

Committed policy and configuration files are always meaningful. `ignored_paths` cannot be used to hide a change that disables or weakens SDD enforcement. Pull-request workflows must check out full history (`fetch-depth: 0`) so the base policy and meaningful-path diff are available.

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
| `exportLevel` | `string?` | `"member"` | Export validation depth: `"type"` (classes/structs only) or `"member"` (all public symbols) |
| `modules` | `object?` | `{}` | Custom module definitions mapping module names to `{ files, depends_on }` |
| `rules` | `object?` | `{}` | Custom validation rules (see [Validation Rules](#validation-rules) below) |
| `taskArchiveDays` | `number?` | — | Days after which completed tasks in companion `tasks.md` files are auto-archived |
| `github` | `object?` | — | GitHub integration settings (see [GitHub Config](#github-config) below) |

---

## Agent-Native Enrichment

SpecSync configuration has no provider, model, API-key, endpoint, timeout, or AI-command fields. `specsync generate` writes deterministic templates without network inference. Use `specsync agents install` or `specsync mcp` to let an existing coding agent enrich those files under that agent's own credentials and permissions. Legacy AI key names are ignored with migration guidance and their values are never printed or executed.

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
