---
module: exports
version: 1
status: stable
files:
  - src/exports/mod.rs
  - src/exports/typescript.rs
  - src/exports/python.rs
  - src/exports/rust_lang.rs
  - src/exports/go.rs
  - src/exports/java.rs
  - src/exports/kotlin.rs
  - src/exports/swift.rs
  - src/exports/dart.rs
  - src/exports/csharp.rs
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
| `extract_exports` | `content: &str` | `Vec<String>` | Per-language backend function that parses source text and returns exported symbol names (one per backend file) |

### Language Backend Functions

Each language backend exposes a single `extract_exports(content: &str) -> Vec<String>` function that parses source code and returns exported symbol names. These are internal to the exports module (not re-exported) and called by `get_exported_symbols`.

| Backend | File | Extraction Strategy |
|---------|------|-------------------|
| TypeScript/JS | `typescript.rs` | `export function/class/interface/type/const/enum`, re-exports (`export { }`, `export type { }`) with `as` alias support; strips `//` and `/* */` comments |
| Python | `python.rs` | Uses `__all__` list if present; otherwise top-level `def`/`class`/`async def` not prefixed with `_` |
| Rust | `rust_lang.rs` | `pub fn/struct/enum/trait/type/const/static/mod` including `pub(crate)` and `pub async/unsafe`; strips comments |
| Go | `go.rs` | Top-level `func/type/var/const` starting with uppercase letter; also exported methods `func (receiver) Name()`; strips comments |
| Java | `java.rs` | `public class/interface/enum/record/@interface` types and `public` methods/fields; handles `static`, `final`, `abstract`, `sealed` modifiers |
| Kotlin | `kotlin.rs` | All top-level `fun/class/object/interface/typealias/val/var/enum class/data class/sealed class` unless marked `private`/`internal`/`protected`; handles `suspend`/`inline` modifiers |
| Swift | `swift.rs` | `public`/`open` declarations: `func/class/struct/enum/protocol/typealias/var/let/actor`; detects `public init` separately; handles `static class func` |
| Dart | `dart.rs` | `class/mixin/enum/extension/typedef` types, `final`/`const` declarations, top-level functions; excludes `_`-prefixed private identifiers |
| C# | `csharp.rs` | `public class/struct/interface/enum/record/delegate` types and `public` members; handles `static`, `partial`, `sealed`, `abstract`, `virtual`, `override`, `async` modifiers |

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
11. Swift backend distinguishes `public` and `open` visibility (both are exported)
12. Kotlin treats everything as public by default unless marked `private`/`internal`/`protected`
13. Dart treats everything as public by default unless prefixed with `_`
14. Java and C# backends require explicit `public` keyword for exports
15. All backends strip single-line (`//`) and multi-line (`/* */`) comments before extraction (except Python which doesn't use this pattern)
16. Go backend deduplicates methods that might also match top-level declarations

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

### Scenario: Python __all__ takes precedence

- **Given** a `.py` file with `__all__ = ["create_auth", "AuthService"]` and additional top-level functions
- **When** `get_exported_symbols(path)` is called
- **Then** returns only the symbols listed in `__all__`, not all top-level definitions

### Scenario: Go uppercase convention

- **Given** a `.go` file with `func CreateAuth()` and `func privateHelper()`
- **When** `get_exported_symbols(path)` is called
- **Then** includes "CreateAuth" but not "privateHelper"

### Scenario: Kotlin default visibility

- **Given** a `.kt` file with `fun publicFun()` and `private fun privateFun()`
- **When** `get_exported_symbols(path)` is called
- **Then** includes "publicFun" (public by default) but not "privateFun"

### Scenario: TypeScript re-exports with aliases

- **Given** a `.ts` file with `export { Foo as Bar }`
- **When** `get_exported_symbols(path)` is called
- **Then** includes "Bar" (the alias), not "Foo"

### Scenario: Comments are stripped before extraction

- **Given** a `.ts` file with `// export function notExported()` inside a comment
- **When** `get_exported_symbols(path)` is called
- **Then** does not include "notExported"

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
