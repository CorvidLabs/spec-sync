---
module: cmd_import
version: 4
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

Implements the `specsync import` command. Imports specs from external systems (GitHub Issues, Jira, Confluence) and local Markdown directories. Directory imports preserve complete valid specs byte-for-byte, augment incomplete documents without discarding their content, and reject malformed frontmatter before writing output.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_import` | `root: &Path, source: Option<&str>, id: Option<&str>, repo_override: Option<&str>, all_issues: bool, label: Option<&str>, from_dir: Option<&Path>` | `()` | Route external data into specs: a single GitHub/Jira/Confluence item, all open issues (`all_issues`/`label`), or a directory of files (`from_dir`) |

## Invariants

1. Supported sources: `github`, `jira`, `confluence`
2. GitHub import resolves repo from config, CLI flag, or git remote
3. Creates spec and companion files (tasks.md, context.md, requirements.md, testing.md); design.md is generated only when `companions.design` is enabled in config
4. Will not overwrite existing spec
5. Success guidance tells users to validate and complete imported details, not to fill template markers
6. Directory imports never report success for output with an empty `files` list: source code is auto-detected, with the canonically confined project-relative input document used as the ownership fallback
7. A complete valid spec keeps its original bytes and declared `module`; incomplete content is only supplemented with missing frontmatter fields and required sections
8. Duplicate or wrongly shaped known frontmatter fields fail the affected import without creating its spec

## Behavioral Examples

### Scenario: Import GitHub issue

- **Given** `specsync import github 42`
- **When** `cmd_import` runs
- **Then** fetches issue #42, creates spec from its title and body

### Scenario: Import an existing spec

- **Given** `docs/renamed.spec.md` is a complete valid spec declaring `module: auth`
- **When** `specsync import --from-dir docs` runs
- **Then** creates `specs/auth/auth.spec.md` with bytes identical to the source document

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Invalid source type | Exits 1 with supported list |
| Spec already exists | Exits 1 |
| Fetch fails | Exits 1 with error |
| Empty file, unterminated frontmatter, duplicate key, or known-field shape mismatch | Counts an error, creates no spec for that file, and exits 1 after the batch |
| No source code match and the input document is outside the project root | Counts an error rather than writing `files: []` |

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
| 2026-07-26 | Preserve valid directory-imported specs byte-for-byte, retain declared module identity, provide non-empty source ownership, and reject malformed frontmatter before output (#416) |
| 2026-06-07 | Update import success guidance for guided imported specs |
| 2026-04-09 | Initial spec |
| 2026-04-13 | Document testing.md and conditional design.md in companion generation |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
