---
module: exports
version: 1
status: stable
files:
  - src/exports/mod.rs
db_tables: []
depends_on:
  - specs/types/types.spec.md
---

# Exports

## Purpose

Language-aware export extraction from source files. Auto-detects the programming language from file extension and extracts public/exported symbol names using regex-based parsing (no AST required). Supports 9 languages: TypeScript/JS, Rust, Go, Python, Swift, Kotlin, Java, C#, and Dart.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `get_exported_symbols` | `file_path: &Path` | `Vec<String>` | Extract exported symbol names from a source file, auto-detecting language from extension |
| `is_test_file` | `file_path: &Path` | `bool` | Check if a file is a test file based on language-specific naming conventions |
| `is_source_file` | `file_path: &Path` | `bool` | Check if a file extension belongs to a supported source language |
| `has_extension` | `file_path: &Path, extensions: &[String]` | `bool` | Check if file matches specific extensions, or any supported language if extensions is empty |

## Invariants

1. Language detection is purely extension-based — no content inspection needed
2. Symbols are deduplicated while preserving order
3. Unreadable files or unknown extensions return an empty vector (never panic)
4. `has_extension` with an empty extensions list delegates to `is_source_file` (matches all supported languages)
5. Test file detection uses language-specific patterns (e.g. `.test.ts`, `_test.go`, `Test.java`)
6. Each language backend uses `LazyLock<Regex>` for compiled patterns — compiled once, reused across calls
7. TypeScript backend handles `export function/class/type/const/enum/interface` and re-exports
8. Rust backend extracts `pub fn/struct/enum/trait/type/const/static/mod` items
9. Go backend extracts uppercase (exported) identifiers and methods
10. Python backend uses `__all__` if present, otherwise top-level non-underscore `def/class`

## Behavioral Examples

### Scenario: Extract TypeScript exports

- **Given** a `.ts` file containing `export function authenticate(token: string): User`
- **When** `get_exported_symbols(path)` is called
- **Then** includes "authenticate" in the returned vector

### Scenario: Extract Rust pub items

- **Given** a `.rs` file containing `pub fn validate_spec(...)`
- **When** `get_exported_symbols(path)` is called
- **Then** includes "validate_spec" in the returned vector

### Scenario: Unsupported file type

- **Given** a `.rb` (Ruby) file
- **When** `get_exported_symbols(path)` is called
- **Then** returns an empty vector

### Scenario: Test file detection

- **Given** a file named `auth.test.ts`
- **When** `is_test_file(path)` is called
- **Then** returns `true`

## Error Cases

| Condition | Behavior |
|-----------|----------|
| File cannot be read | Returns empty vector |
| Unknown file extension | Returns empty vector |
| File has no exports | Returns empty vector |
| Binary or non-text file | Returns empty vector (read_to_string fails gracefully) |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| types | `Language` enum for extension-to-language mapping |

### Consumed By

| Module | What is used |
|--------|-------------|
| validator | `get_exported_symbols`, `has_extension`, `is_test_file` |
| scoring | `get_exported_symbols` |
| generator | `has_extension`, `is_test_file` |
| config | `has_extension` |

## Change Log

| Date | Change |
|------|--------|
| 2026-03-25 | Initial spec |
