---
module: module-name
version: 1
status: draft
files:
  - src/path/to/file.ts
db_tables: []
depends_on: []
---

# Module Name

## Purpose

<!-- Plain English: what this module does and why it exists -->

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `example` | `(id: string)` | `Thing \| null` | Fetches a thing by ID |

### Exported Types

| Type | Description |
|------|-------------|
| `ExampleType` | Represents a thing |

### Exported Classes

| Class | Description |
|-------|-------------|
| `ExampleService` | Manages things |

#### ExampleService Methods

| Method | Parameters | Returns | Description |
|--------|-----------|---------|-------------|
| `doThing` | `(id: string)` | `Promise<void>` | Does the thing |

## Invariants

1. Example invariant that must always be true

## Behavioral Examples

### Scenario: Example scenario

- **Given** some precondition
- **When** an action occurs
- **Then** this result is expected

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Thing not found | Returns null |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|

### Consumed By

| Module | What is used |
|--------|-------------|

## Change Log

| Date | Author | Change |
|------|--------|--------|
| YYYY-MM-DD | name | Initial spec |
