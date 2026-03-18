# SpecSync

Bidirectional spec-to-code validation. Keep your module specs and source code in sync with CI-enforced contract checking.

## What it does

SpecSync validates that your markdown module specifications match your actual TypeScript source code — in both directions:

- **Code exports something not in the spec?** Warning: undocumented export
- **Spec documents something that doesn't exist?** Error: stale spec entry
- **Source file referenced in spec was deleted?** Error: missing file
- **DB table in spec doesn't exist in schema?** Error: phantom table
- **Required section missing from spec?** Error: incomplete spec

## Install

```bash
# bun
bun add -d specsync

# npm
npm install --save-dev specsync
```

## Quick start

```bash
# Create a config file
specsync init

# Validate all specs
specsync check

# See coverage report
specsync coverage

# Generate specs for unspecced modules
specsync generate
```

## Spec format

Specs are markdown files with YAML frontmatter:

```markdown
---
module: auth
version: 3
status: stable
files:
  - src/auth/service.ts
  - src/auth/middleware.ts
db_tables:
  - users
  - sessions
depends_on:
  - specs/database/database.spec.md
---

# Auth

## Purpose

Handles authentication and session management.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `authenticate` | `(token: string)` | `User \| null` | Validates a token |

### Exported Types

| Type | Description |
|------|-------------|
| `User` | Authenticated user object |

## Invariants

1. Sessions expire after 24 hours
2. Failed auth attempts are rate-limited

## Behavioral Examples

### Scenario: Valid token

- **Given** a valid JWT token
- **When** `authenticate()` is called
- **Then** returns the corresponding User

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Expired token | Returns null, logs warning |
| Malformed token | Returns null |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| database | `query()` for user lookups |

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-03-18 | team | Initial spec |
```

## Configuration

Create a `specsync.json` in your project root:

```json
{
  "specsDir": "specs",
  "sourceDirs": ["src"],
  "schemaDir": "db/migrations",
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
  "excludePatterns": ["**/__tests__/**", "**/*.test.ts", "**/*.spec.ts"]
}
```

| Option | Default | Description |
|--------|---------|-------------|
| `specsDir` | `"specs"` | Directory containing spec files |
| `sourceDirs` | `["src"]` | Source directories for coverage |
| `schemaDir` | — | SQL schema dir for `db_tables` validation |
| `schemaPattern` | `CREATE TABLE` regex | Pattern to extract table names |
| `requiredSections` | Standard set | Required markdown sections |
| `excludeDirs` | `["__tests__"]` | Dirs excluded from coverage |
| `excludePatterns` | test files | File patterns excluded from coverage |

## CLI

```
specsync [command] [flags]

Commands:
  check       Validate all specs against source (default)
  coverage    Show file and module coverage report
  generate    Scaffold specs for unspecced modules
  init        Create specsync.json config file
  help        Show help

Flags:
  --strict              Treat warnings as errors
  --require-coverage N  Fail if file coverage < N%
  --root <path>         Project root (default: cwd)
```

## CI integration

```yaml
# GitHub Actions
- name: Spec check
  run: npx specsync check --strict --require-coverage 100
```

## How it works

1. **Discovers** all `*.spec.md` files in your specs directory
2. **Parses** YAML frontmatter (zero-dependency regex parser, no YAML library)
3. **Validates structure** — required fields, required sections, file existence
4. **Validates API surface** — parses TypeScript exports and cross-references against spec's Public API tables
5. **Validates dependencies** — checks that `depends_on` spec files exist
6. **Reports coverage** — which source files and modules have specs

## License

MIT
