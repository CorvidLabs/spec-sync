---
module: ignore
version: 8
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

**Exported Enums**

| Type | Description |
|------|-------------|
| `WarningCategory` | Classifiable warning types, including the partial `N/M exports documented` summary |

**Exported Structs**

| Type | Description |
|------|-------------|
| `IgnoreRules` | Global and per-spec suppression rules plus deterministic load diagnostics |

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `as_str` | Alias for `WarningCategory::as_str` | `&'static str` | Return the stable machine-readable category name |
| `from_str` | Alias for `WarningCategory::from_str` | `Option<WarningCategory>` | Parse category name string |
| `classify` | Alias for `WarningCategory::classify` | `Option<WarningCategory>` | Classify warning text |
| `load` | Alias for `IgnoreRules::load` | `IgnoreRules` | Load from `.specsyncignore` |
| `parse_inline` | Alias for `IgnoreRules::parse_inline` | `HashSet<WarningCategory>` | Parse inline directives |
| `is_suppressed` | Alias for `IgnoreRules::is_suppressed` | `bool` | Check suppression |
| `suppression_source` | Alias for `IgnoreRules::suppression_source` | `Option<(WarningCategory, &'static str)>` | Return the matched category and global/inline/path source |

## Invariants

1. Suppression is checked in order: global → inline → per-spec path prefix match
2. `.specsyncignore` accepts both `category:path` and `path:category`, uses `#` for comments, strips a leading UTF-8 BOM, and diagnoses invalid UTF-8 per line
3. Per-spec rules match by path prefix — `stub-section:specs/legacy/` suppresses for all specs under that directory
4. `classify()` checks `SchemaTypeMismatch` before `SchemaColumn` to prevent the more general pattern from shadowing the specific one
5. `classify()` maps both legacy section-stub wording and current unfinished-draft wording to `StubSection`
6. `from_str()` normalizes underscores to hyphens and lowercases before matching, supporting both `requirements_companion` and `requirements-companion`
7. Missing `.specsyncignore` file is not an error — returns empty rules
8. Unrecognized category names and malformed rules remain visible through `IgnoreRules::warnings`
9. Per-spec matching normalizes forward/backward separators and an optional `./` prefix before prefix comparison

## Behavioral Examples

### Scenario: Global suppression

- **Given** `.specsyncignore` contains `requirements-companion`
- **When** a spec triggers "Missing companion requirements.md" warning
- **Then** `is_suppressed()` returns true for any spec path

### Scenario: Per-spec path suppression

- **Given** `.specsyncignore` contains `stub-section:specs/legacy/`
- **When** spec `specs/legacy/api.spec.md` has a Purpose section with no substantive content
- **Then** warning is suppressed
- **But** spec `specs/core/core.spec.md` with an empty Purpose section is NOT suppressed

### Scenario: Inline directive

- **Given** spec body contains `<!-- specsync-ignore: undocumented-export, changelog -->`
- **When** `parse_inline()` is called
- **Then** returns set containing `UndocumentedExport` and `ChangelogEntries`

### Scenario: Corrupt line does not disable valid rules

- **Given** `.specsyncignore` contains valid UTF-8 rules before and after one invalid UTF-8 line
- **When** `IgnoreRules::load()` runs
- **Then** both valid rules apply and the invalid line produces a numbered diagnostic

## Error Cases

| Condition | Behavior |
|-----------|----------|
| `.specsyncignore` does not exist | Returns empty `IgnoreRules` (not an error) |
| Unrecognized category string | `from_str()` returns `None`; file loading records a numbered diagnostic |
| Invalid UTF-8 line | That line is skipped with a numbered diagnostic; other lines still load |
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
| 2026-06-07 | Teach warning classification about unfinished-draft section wording |
| 2026-04-09 | Initial spec |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-11 | CHG-0013-preserve-punctuated-public-api-symbols-across-all-export-extractors: Move associated methods under the informational method subsection so exact symbol parsing validates only real module exports |
| 2026-07-11 | CHG-0013-preserve-punctuated-public-api-symbols-across-all-export-extractors: Preserve punctuated Public API symbols across all export extractors |
| 2026-07-30 | CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara: Stabilize SpecSync 6.0 with one scope approval, same-PR finalization, lightweight archive CI, scoped review, and selected UX fixes |
| 2026-07-31 | CHG-0070-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes: Land pre-6.0 product fixes for hooks init coverage naming and exit codes |
| 2026-08-01 | CHG-0071-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes-scoped: Land pre-6.0 product fixes for hooks init coverage naming and exit codes (scoped paths) |
