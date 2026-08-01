---
module: util
version: 3
status: stable
files:
  - src/util.rs
db_tables: []
depends_on: []
---

# Util

## Purpose

Provides small shared utility functions used by validators, fix suggestions, and rule compilation. The module keeps low-level helpers centralized so parsing and validation modules do not duplicate edit-distance or regex-safety logic.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `levenshtein` | `a: &str, b: &str` | `usize` | Computes Unicode-scalar edit distance between two strings |
| `safe_regex` | `pattern: &str` | `Option<regex::Regex>` | Compiles a user-provided regex with bounded regex and DFA size limits |
| `confine_path_to_root` | `root, rel` | `Option<PathBuf>` | Resolve a project-relative path and reject escapes outside the root |

## Invariants

1. `levenshtein` treats input as `char` sequences rather than raw bytes.
2. `levenshtein` returns `0` for equal strings and the other string's character length when one side is empty.
3. `safe_regex` never panics on invalid user input; invalid or oversized patterns return `None`.
4. `safe_regex` applies the same maximum limit to compiled regex size and DFA size.

## Behavioral Examples

### Scenario: Suggest nearby filenames

- **Given** the strings `config.ts` and `confg.ts`
- **When** `levenshtein` compares them
- **Then** it returns `1`, allowing validation to suggest the near miss

### Scenario: Reject invalid regex

- **Given** an invalid pattern such as `[invalid`
- **When** `safe_regex` tries to compile it
- **Then** it returns `None` instead of propagating a regex parser error

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Empty string passed to `levenshtein` | Returns the character length of the other string |
| Invalid regex syntax | `safe_regex` returns `None` |
| Pattern exceeds configured regex size limits | `safe_regex` returns `None` |

## Dependencies

### Consumes

| Module | What is used |
|--------|--------------|
| regex | `RegexBuilder` for bounded regex compilation |

### Consumed By

| Module | What is used |
|--------|--------------|
| validator | `levenshtein`, `safe_regex` |
| parser | `levenshtein` |

## Change Log

| Date | Change |
|------|--------|
| 2026-06-07 | Initial spec for shared utility helpers |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-08-01 | CHG-0071-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes-scoped: Land pre-6.0 product fixes for hooks init coverage naming and exit codes (scoped paths) |
