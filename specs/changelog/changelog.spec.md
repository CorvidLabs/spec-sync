---
module: changelog
version: 1
status: stable
files:
  - src/changelog.rs
db_tables: []
tracks: [141]
depends_on:
  - specs/parser/parser.spec.md
  - specs/types/types.spec.md
---

# Changelog

## Purpose

Automated changelog generation for spec changes between git refs. Compares specs at two git commits/tags and produces a structured diff showing which specs were added, removed, or modified — and which specific fields changed for modified specs.

## Public API

### Exported Structs

| Type | Description |
|------|-------------|
| `FieldChange` | A single field change within a modified spec (field name, old value, new value) |
| `ModifiedSpec` | A spec that was modified between two refs, with its list of field changes |
| `ChangelogReport` | The full changelog comparing two git refs — added, removed, and modified specs |
| `SpecEntry` | A spec entry for added/removed lists (module name, path, status, version) |

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `parse_range` | `range: &str` | `Option<(String, String)>` | Parse a range string like "v0.1..v0.2" into (from, to) tuple |
| `generate_changelog` | `root, specs_dir, from_ref, to_ref` | `ChangelogReport` | Generate a changelog comparing specs between two git refs |
| `format_text` | `report: &ChangelogReport` | `String` | Format changelog as colored terminal text |
| `format_json` | `report: &ChangelogReport` | `String` | Format changelog as JSON |
| `format_markdown` | `report: &ChangelogReport` | `String` | Format changelog as markdown |

## Invariants

1. Spec comparison uses git `ls-tree` and `show` to read spec state at each ref — never touches the working tree
2. Field-level diffing compares frontmatter fields: module, version, status, files, db_tables, depends_on
3. `parse_range` only accepts `..` as separator and requires non-empty parts on both sides
4. Added/removed/modified lists are sorted by spec path (BTreeMap/BTreeSet ordering)
5. A spec is "modified" only if at least one frontmatter field changed between the two refs
6. Output formatters (text, json, markdown) share the same ChangelogReport — formatting is decoupled from analysis

## Behavioral Examples

### Scenario: Generate changelog between two tags

- **Given** specs at `v1.0.0` and `v1.1.0` where `auth.spec.md` was added and `config.spec.md` had its version bumped
- **When** `generate_changelog(root, "specs", "v1.0.0", "v1.1.0")` is called
- **Then** returns a report with auth in `added` and config in `modified` with a version FieldChange

### Scenario: Parse valid range

- **Given** the string "v1.0..v2.0"
- **When** `parse_range("v1.0..v2.0")` is called
- **Then** returns `Some(("v1.0", "v2.0"))`

### Scenario: Parse invalid range

- **Given** the string "v1.0"
- **When** `parse_range("v1.0")` is called
- **Then** returns `None`

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Git ref doesn't exist | `list_specs_at_ref` returns empty list — no crash |
| Range string has no `..` separator | `parse_range` returns `None` |
| Spec frontmatter unparseable at a ref | Spec is silently skipped in diff |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| parser | `parse_frontmatter` |
| types | `Frontmatter` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli | `generate_changelog`, `parse_range`, formatters |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-07 | Initial spec |
