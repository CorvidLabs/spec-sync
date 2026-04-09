---
module: cmd_resolve
version: 1
status: stable
files:
  - src/commands/resolve.rs
db_tables: []
tracks: []
depends_on:
  - specs/commands/commands.spec.md
  - specs/parser/parser.spec.md
  - specs/registry/registry.spec.md
  - specs/validator/validator.spec.md
---

# Cmd Resolve

## Purpose

Implements the `specsync resolve` command. Resolves dependency references — local by file existence, cross-project via registry lookups. `--remote` fetches remote registries from GitHub.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_resolve` | `root: &Path, remote: bool` | `()` | Resolve all dependency references; optionally fetch remote registries |

## Invariants

1. Classifies deps as local vs cross-project
2. Local verified by file existence
3. No network calls without `--remote`
4. Warnings for unresolvable refs (does not exit non-zero)

## Behavioral Examples

### Scenario: All local deps resolve

- **Given** all `depends_on` point to existing files
- **When** `cmd_resolve(root, false)` runs
- **Then** prints "All N dependencies resolved"

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Local dep missing | Warning printed |
| Remote fetch fails | Warning, continues |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| commands | `load_and_discover` |
| parser | `parse_frontmatter` |
| registry | `fetch_remote_registry` |
| validator | `find_spec_files` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync resolve` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
