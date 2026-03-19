---
title: Spec Format
layout: default
nav_order: 2
---

# Spec Format
{: .no_toc }

Specs are markdown files (`*.spec.md`) with YAML frontmatter. Place them in your specs directory (default: `specs/`).
{: .fs-6 .fw-300 }

<details open markdown="block">
  <summary>Table of contents</summary>
  {: .text-delta }
1. TOC
{:toc}
</details>

---

## Frontmatter

Every spec file starts with YAML frontmatter between `---` delimiters:

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
| `module` | `string` | Module name — used for display and identification |
| `version` | `number` | Spec version — increment when the spec changes |
| `status` | `enum` | One of: `draft`, `review`, `stable`, `deprecated` |
| `files` | `string[]` | Source files this spec covers (must be non-empty) |

### Optional Fields

| Field | Type | Description |
|:------|:-----|:------------|
| `db_tables` | `string[]` | Database tables used by this module — validated against your schema directory |
| `depends_on` | `string[]` | Paths to other spec files this module depends on — validated for existence |

---

## Required Sections

By default, every spec must include these markdown sections (configurable via `requiredSections` in `specsync.json`):

| Section | Purpose |
|:--------|:--------|
| `## Purpose` | What this module does and why it exists |
| `## Public API` | Tables listing exported symbols — **this is what gets validated against code** |
| `## Invariants` | Rules that must always hold true |
| `## Behavioral Examples` | Given/When/Then scenarios showing expected behavior |
| `## Error Cases` | How the module handles failure conditions |
| `## Dependencies` | What this module consumes from other modules |
| `## Change Log` | History of spec changes |

You can customize these in your config:

```json
{
  "requiredSections": ["Purpose", "Public API", "Change Log"]
}
```

---

## Public API Tables

The Public API section is the core of what SpecSync validates. Use markdown tables with **backtick-quoted symbol names** — SpecSync extracts the first backtick-quoted identifier per table row and cross-references it against actual code exports.

```markdown
## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `authenticate` | `(token: string)` | `User \| null` | Validates a bearer token |
| `refreshSession` | `(sessionId: string)` | `Session` | Extends session TTL |

### Exported Types

| Type | Description |
|------|-------------|
| `User` | Authenticated user object |
| `Session` | Active session record |
```

{: .note }
> The table column headers don't matter — SpecSync only looks at backtick-quoted names in the first column. You can structure the table however works best for your team.

---

## Consumed By Section

You can optionally track reverse dependencies in a `### Consumed By` subsection under Dependencies. SpecSync validates that referenced files actually exist:

```markdown
## Dependencies

### Consumed By

| Module | What is used |
|--------|-------------|
| api-gateway | Uses `authenticate()` middleware |
```

---

## Full Example

Here's a complete spec file showing all features:

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
manages session lifecycle, and provides middleware for route protection.

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
2. Failed auth attempts are rate-limited to 5/minute
3. Tokens are validated cryptographically, never by string comparison

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

### Consumes

| Module | What is used |
|--------|-------------|
| database | `query()` for user lookups |
| crypto | `verifyJwt()` for token validation |

### Consumed By

| Module | What is used |
|--------|-------------|
| api-gateway | Uses `authenticate()` middleware |

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-03-18 | team | Initial spec |
```

---

## Custom Templates

When running `specsync generate`, you can provide a custom template at `specs/_template.spec.md`. If no template exists, SpecSync uses a built-in default with all seven required sections and TODO placeholders.

The generator replaces these frontmatter fields automatically:
- `module:` — set to the module directory name
- `version:` — set to `1`
- `status:` — set to `draft`
- `files:` — populated with discovered source files
