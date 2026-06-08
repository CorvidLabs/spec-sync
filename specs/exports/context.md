---
spec: exports.spec.md
---

## Key Decisions

- **Regex by default, AST opt-in**: All 12 language backends use compiled regexes (`LazyLock<Regex>`). `ParseMode::Ast` enables tree-sitter backends for TypeScript, Python, and Rust only; if the AST backend returns nothing (parse failure), `mod.rs` falls back to the regex backend automatically.
- **Comment stripping first**: Every backend strips string literals then comments before running export regexes, preventing false matches inside strings or commented-out code.
- **TypeScript wildcard resolution**: `export * from './foo'` is resolved one level deep via `resolve_ts_import` to prevent infinite re-export loops. Namespace re-exports (`export * as Ns`) produce the namespace name, not the inner symbols. Without a resolver, wildcard lines are skipped.
- **Python `__all__` precedence**: If `__all__` is defined, it is the sole source of exports. Otherwise, top-level non-underscore functions and classes are extracted.
- **Go capitalization convention**: Uppercase first letter = exported. Methods are extracted separately from functions.
- **Two export levels**: `ExportLevel::Type` (via `filter_type_level_exports`) extracts only top-level type declarations (class, struct, enum, interface); `ExportLevel::Member` extracts all public symbols. This allows specs to document at the right granularity.

## Files to Read First

- `src/exports/mod.rs` — Router, language detection, `get_exported_symbols()`/`get_exported_symbols_full()` entry points, `filter_type_level_exports`, `resolve_ts_import`, and `is_test_file()`/`is_source_file()`/`has_extension()` helpers.
- `src/exports/typescript.rs` — Most complex regex backend (re-exports, wildcards, defaults). Good reference for understanding the pattern.
- `src/exports/ast/tests.rs` — Parity tests asserting AST backends match the regex backends.

## Current Status

Fully implemented for all 12 languages: TypeScript/JS, Rust, Go, Python, Swift, Kotlin, Java, C#, Dart, PHP, Ruby, YAML. Each regex backend has compiled patterns; AST backends exist for TypeScript, Python, and Rust under `src/exports/ast/`.

## Notes

- This is the only multi-file module in the project (~14 source files under `src/exports/`, plus the `ast/` subdirectory).
- Language detection is purely extension-based — no shebang or content sniffing.
- The validator uses this module bidirectionally: undocumented exports = warning, documented-but-missing exports = error.
