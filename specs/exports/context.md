---
spec: exports.spec.md
---

## Key Decisions

- **Regex over AST**: All 9 language backends use compiled regexes (`LazyLock<Regex>`) rather than language-specific AST parsers. This keeps the binary small and avoids per-language parser dependencies at the cost of some edge-case accuracy.
- **Comment stripping first**: Every backend strips string literals then comments before running export regexes, preventing false matches inside strings or commented-out code.
- **TypeScript wildcard resolution**: `export * from './foo'` is resolved one level deep to prevent infinite re-export loops. Namespace re-exports (`export * as Ns`) produce the namespace name, not the inner symbols.
- **Python `__all__` precedence**: If `__all__` is defined, it is the sole source of exports. Otherwise, top-level non-underscore functions and classes are extracted.
- **Go capitalization convention**: Uppercase first letter = exported. Methods are extracted separately from functions.
- **Two export levels**: `ExportLevel::Type` extracts only top-level type declarations (class, struct, enum, interface); `ExportLevel::Member` extracts all public symbols. This allows specs to document at the right granularity.

## Files to Read First

- `src/exports/mod.rs` — Router, language detection, `get_exported_symbols()` entry point, and `is_test_file()`/`is_source_file()` helpers.
- `src/exports/typescript.rs` — Most complex backend (re-exports, wildcards, defaults). Good reference for understanding the pattern.

## Current Status

Fully implemented for all 9 languages: TypeScript/JS, Rust, Go, Python, Swift, Kotlin, Java, C#, Dart. Each backend has compiled regex patterns and handles language-specific idioms.

## Notes

- This is the only multi-file module in the project (10 source files under `src/exports/`).
- Language detection is purely extension-based — no shebang or content sniffing.
- The validator uses this module bidirectionally: undocumented exports = warning, documented-but-missing exports = error.
