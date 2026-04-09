---
module: ignore
version: 1
status: stable
files:
  - src/ignore.rs
db_tables: []
tracks: []
depends_on: []
---

# Ignore

## Purpose

Provides a warning suppression system for spec-sync validation. Supports three layers of suppression: global rules via `.specsyncignore` file, per-spec path rules in the same file, and inline directives in spec markdown via HTML comments. Allows teams to intentionally silence known warnings without fixing them.

## Public API

### Exported Enums

| Type | Description |
|------|-------------|
| `WarningCategory` | 13-variant enum representing classifiable warning types: RequirementsCompanion, StubSection, UndocumentedExport, Deprecated, UnknownStatus, UnknownAgentPolicy, SchemaColumn, SchemaTypeMismatch, ConsumedBy, ChangelogEntries, SpecSize, MinInvariants, RequireDependsOn |

### Exported Structs

| Type | Description |
|------|-------------|
| `IgnoreRules` | Container holding global suppression set and per-spec suppression map, loaded from `.specsyncignore` |

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `WarningCategory::from_str` | `s: &str` | `Option<Self>` | Parse category from string (case-insensitive, kebab-case, with aliases like "requirements" → RequirementsCompanion) |
| `WarningCategory::classify` | `warning: &str` | `Option<Self>` | Classify a warning message text into a category by pattern matching on known prefixes/keywords |
| `IgnoreRules::load` | `root: &Path` | `Self` | Load rules from `.specsyncignore` file; returns empty rules if file doesn't exist |
| `IgnoreRules::parse_inline` | `body: &str` | `HashSet<WarningCategory>` | Extract inline ignore directives from spec body (`<!-- specsync-ignore: cat1, cat2 -->`) |
| `IgnoreRules::is_suppressed` | `&self, warning, spec_rel_path, inline_ignores` | `bool` | Check if a warning should be suppressed given global, inline, and per-spec rules |
| `from_str` | Alias for `WarningCategory::from_str` | `Option<WarningCategory>` | Parse category name string |
| `classify` | Alias for `WarningCategory::classify` | `Option<WarningCategory>` | Classify warning text |
| `load` | Alias for `IgnoreRules::load` | `IgnoreRules` | Load from `.specsyncignore` |
| `parse_inline` | Alias for `IgnoreRules::parse_inline` | `HashSet<WarningCategory>` | Parse inline directives |
| `is_suppressed` | Alias for `IgnoreRules::is_suppressed` | `bool` | Check suppression |

## Invariants

1. Suppression is checked in order: global → inline → per-spec path prefix match
2. `.specsyncignore` uses `#` for comments (line and inline) and `:` to separate category from path
3. Per-spec rules match by path prefix — `stub-section:specs/legacy/` suppresses for all specs under that directory
4. `classify()` checks `SchemaTypeMismatch` before `SchemaColumn` to prevent the more general pattern from shadowing the specific one
5. `from_str()` normalizes underscores to hyphens and lowercases before matching, supporting both `requirements_companion` and `requirements-companion`
6. Missing `.specsyncignore` file is not an error — returns empty rules
7. Unrecognized category names in `.specsyncignore` are silently ignored (no error)

## Behavioral Examples

### Scenario: Global suppression

- **Given** `.specsyncignore` contains `requirements-companion`
- **When** a spec triggers "Missing companion requirements.md" warning
- **Then** `is_suppressed()` returns true for any spec path

### Scenario: Per-spec path suppression

- **Given** `.specsyncignore` contains `stub-section:specs/legacy/`
- **When** spec `specs/legacy/api.spec.md` has a stub Purpose section
- **Then** warning is suppressed
- **But** spec `specs/core/core.spec.md` with stub Purpose is NOT suppressed

### Scenario: Inline directive

- **Given** spec body contains `<!-- specsync-ignore: undocumented-export, changelog -->`
- **When** `parse_inline()` is called
- **Then** returns set containing `UndocumentedExport` and `ChangelogEntries`

## Error Cases

| Condition | Behavior |
|-----------|----------|
| `.specsyncignore` does not exist | Returns empty `IgnoreRules` (not an error) |
| Unrecognized category string | Silently skipped during load; `from_str()` returns `None` |
| Malformed inline comment (missing `-->`) | Directive is ignored |
| Warning text doesn't match any pattern | `classify()` returns `None`, warning is never suppressed |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| std::fs | Read `.specsyncignore` file |
| std::collections | HashSet, HashMap for rule storage |

### Consumed By

| Module | What is used |
|--------|-------------|
| commands (mod.rs) | `IgnoreRules::load()` and `is_suppressed()` in validation pipeline |
| cmd_check | `IgnoreRules` for filtered validation |
| cmd_coverage | `IgnoreRules` for filtered validation |
| cmd_generate | `IgnoreRules` for filtered validation |
| cmd_issues | `IgnoreRules` for filtered validation |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
