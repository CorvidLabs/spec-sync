---
title: Spec Format
layout: default
nav_order: 2
---

# Spec Format
{: .no_toc }

Specs are markdown files (`*.spec.md`) with YAML frontmatter, placed in your specs directory (default: `specs/`).
{: .fs-6 .fw-300 }

<details open markdown="block">
  <summary>Table of contents</summary>
  {: .text-delta }
1. TOC
{:toc}
</details>

---

## Frontmatter

```yaml
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
```

### Required Fields

| Field | Type | Description |
|:------|:-----|:------------|
| `module` | `string` | Module name for display and identification |
| `version` | `number` | Increment when the spec changes |
| `status` | `enum` | `draft`, `review`, `stable`, or `deprecated` |
| `files` | `string[]` | Source files this spec covers (must be non-empty) |

### Optional Fields

| Field | Type | Description |
|:------|:-----|:------------|
| `db_tables` | `string[]` | Validated against `CREATE TABLE` statements in your `schemaDir` |
| `depends_on` | `string[]` | Paths to other spec files — validated for existence |

---

## Required Sections

Every spec must include these `## Heading` sections (configurable via `requiredSections` in `specsync.json`):

| Section | What SpecSync checks |
|:--------|:---------------------|
| `## Purpose` | Presence only |
| `## Public API` | Backtick-quoted symbols cross-referenced against code exports |
| `## Invariants` | Presence only |
| `## Behavioral Examples` | Presence only |
| `## Error Cases` | Presence only |
| `## Dependencies` | Presence only |
| `## Change Log` | Presence only |

Override the list in config:

```json
{ "requiredSections": ["Purpose", "Public API"] }
```

---

## Public API Tables

The core of what SpecSync validates. Use markdown tables with **backtick-quoted symbol names** — SpecSync extracts the first backtick-quoted identifier per row and cross-references it against code exports.

```markdown
## Public API

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `authenticate` | `(token: string)` | `User \| null` | Validates bearer token |
```

{: .note }
> Column headers don't matter. SpecSync only reads backtick-quoted names in the first column. Structure the table however suits your team.

---

## Consumed By Section

Track reverse dependencies under `## Dependencies`. SpecSync validates that referenced files exist:

```markdown
## Dependencies

### Consumed By

| Module | Usage |
|--------|-------|
| api-gateway | Uses `authenticate()` middleware |
```

---

## Custom Templates

`specsync generate` uses `specs/_template.spec.md` if present, otherwise a built-in default. The generator auto-fills:
- `module:` — directory name
- `version:` — `1`
- `status:` — `draft`
- `files:` — discovered source files

---

## Full Example

<details markdown="block">
<summary>Complete spec file</summary>

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

Handles authentication and session management. Validates bearer tokens,
manages session lifecycle, provides middleware for route protection.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `authenticate` | `(token: string)` | `User \| null` | Validates a token |
| `refreshSession` | `(sessionId: string)` | `Session` | Extends session TTL |

### Exported Types

| Type | Description |
|------|-------------|
| `User` | Authenticated user object |
| `Session` | Active session record |

## Invariants

1. Sessions expire after 24 hours
2. Failed auth attempts rate-limited to 5/minute
3. Tokens validated cryptographically, never by string comparison

## Behavioral Examples

### Scenario: Valid token

- **Given** a valid JWT token
- **When** `authenticate()` is called
- **Then** returns the corresponding User object

### Scenario: Expired token

- **Given** an expired JWT token
- **When** `authenticate()` is called
- **Then** returns null and logs a warning

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Expired token | Returns null, logs warning |
| Malformed token | Returns null |
| DB unavailable | Throws `ServiceUnavailableError` |

## Dependencies

| Module | Usage |
|--------|-------|
| database | `query()` for user lookups |
| crypto | `verifyJwt()` for token validation |

### Consumed By

| Module | Usage |
|--------|-------|
| api-gateway | Uses `authenticate()` middleware |

## Change Log

| Date | Change |
|------|--------|
| 2026-03-18 | Initial spec |
```

</details>
