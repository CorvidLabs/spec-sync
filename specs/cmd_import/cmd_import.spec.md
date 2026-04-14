---
module: cmd_import
version: 1
status: stable
files:
  - src/commands/import.rs
db_tables: []
tracks: []
depends_on:
  - specs/config/config.spec.md
  - specs/generator/generator.spec.md
  - specs/github/github.spec.md
  - specs/importer/importer.spec.md
---

# Cmd Import

## Purpose

Implements the `specsync import` command. Imports specs from external systems (GitHub Issues, Jira, Confluence) by fetching remote data and converting it into spec files with companions.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_import` | `root: &Path, source: &str, id: &str, repo_override: Option<&str>` | `()` | Fetch external data and create a new spec from it |

## Invariants

1. Supported sources: `github`, `jira`, `confluence`
2. GitHub import resolves repo from config, CLI flag, or git remote
3. Creates spec and companion files (tasks.md, context.md, requirements.md, testing.md); design.md is generated only when `companions.design` is enabled in config
4. Will not overwrite existing spec

## Behavioral Examples

### Scenario: Import GitHub issue

- **Given** `specsync import github 42`
- **When** `cmd_import` runs
- **Then** fetches issue #42, creates spec from its title and body

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Invalid source type | Exits 1 with supported list |
| Spec already exists | Exits 1 |
| Fetch fails | Exits 1 with error |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| config | `load_config` |
| generator | `generate_companion_files` |
| github | `resolve_repo` |
| importer | `import_github_issue`, `import_jira_issue`, `import_confluence_page` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync import` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
| 2026-04-13 | Document testing.md and conditional design.md in companion generation |
