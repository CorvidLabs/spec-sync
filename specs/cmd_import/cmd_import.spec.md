---
module: cmd_import
version: 6
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
| `cmd_import` | `root: &Path, source: Option<&str>, id: Option<&str>, repo_override: Option<&str>, all_issues: bool, label: Option<&str>, from_dir: Option<&Path>` | `()` | Route external data into specs: a single GitHub/Jira/Confluence item, all open issues (`all_issues`/`label`), or a directory of files (`from_dir`) |

## Invariants

1. GitHub import reads never launch a provider subprocess.
2. Single imports use the shared typed issue-detail contract.
3. Batch imports use strict, bounded, complete pagination.
4. Missing tokens, malformed responses, inaccessible repositories, transport failures, timeouts,
   pagination ambiguity, and cap truncation fail closed.
5. Unsafe/reserved/overlong output names create no directory or file.
6. Partial batch success never produces a false-green exit status.

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
| `GITHUB_TOKEN` missing for GitHub import | Exits 1 with explicit-token guidance; does not consult `gh` |
| GitHub issue pagination is malformed, duplicated, or exceeds 100 pages | Batch import fails closed without reporting a truncated issue set |
| Imported item has an unsafe/reserved/overlong module name | Fails before creating its output path; batch continues with later items |
| Any batch item errors | Summary includes imported/skipped/error counts and the command exits 1 |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| config | `load_config` |
| generator | `generate_companion_files` |
| github | `resolve_repo`, strict typed `list_issues` pagination |
| importer | `import_github_issue`, `import_jira_issue`, `import_confluence_page` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync import` |

## Change Log

| Date | Change |
|------|--------|
| 2026-06-07 | Update import success guidance for guided imported specs |
| 2026-04-09 | Initial spec |
| 2026-07-22 | CHG-0063: Require explicit-token in-process GitHub REST reads and fail closed on partial pagination |
| 2026-07-23 | CHG-0063 portable batch follow-up: validate every output name before writes, continue batch processing, and exit nonzero when any item fails |
| 2026-04-13 | Document testing.md and conditional design.md in companion generation |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-27 | CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414: Close independent MCP security review gaps for issue 414 |
