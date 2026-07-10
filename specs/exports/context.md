---
spec: exports.spec.md
---

## Key Decisions

- **Regex by default, AST opt-in**: All language backends use compiled regexes (`LazyLock<Regex>`). `ParseMode::Ast` enables tree-sitter backends for TypeScript, Python, Rust, C, C++, Scala, Erlang, Elixir, Perl, and Lisp/Scheme/Emacs Lisp; if the AST backend returns nothing (parse failure), `mod.rs` falls back to the regex backend automatically. Nim and Crystal have no published tree-sitter grammar crate, so they remain regex-only.
- **Comment stripping first**: Every backend strips string literals then comments before running export regexes, preventing false matches inside strings or commented-out code.
- **TypeScript wildcard resolution**: `export * from './foo'` is resolved one level deep via `resolve_ts_import` to prevent infinite re-export loops. Namespace re-exports (`export * as Ns`) produce the namespace name, not the inner symbols. Without a resolver, wildcard lines are skipped.
- **Python `__all__` precedence**: If `__all__` is defined, it is the sole source of exports. Otherwise, top-level non-underscore functions and classes are extracted.
- **Go capitalization convention**: Uppercase first letter = exported. Methods are extracted separately from functions.
- **Two export levels**: `ExportLevel::Type` (via `filter_type_level_exports`) extracts only top-level type declarations (class, struct, enum, interface); `ExportLevel::Member` extracts all public symbols. This allows specs to document at the right granularity.
- **Modern Kotlin modifier chains**: Kotlin declarations accept repeated modifiers in language-valid order rather than one hard-coded sequence. The regex backend also handles same-line annotations (including one level of nested argument parentheses), `value class`, `expect`/`actual`, and `external`, while restricted visibility still wins even when it follows another modifier. A restricted annotated type opens a non-exportable scope so its public-looking members cannot leak into the module API.

## Files to Read First

- `src/exports/mod.rs` — Router, language detection, `get_exported_symbols()`/`get_exported_symbols_full()` entry points, `filter_type_level_exports`, `resolve_ts_import`, and `is_test_file()`/`is_source_file()`/`has_extension()` helpers.
- `src/exports/typescript.rs` — Most complex regex backend (re-exports, wildcards, defaults). Good reference for understanding the pattern.
- `src/exports/ast/tests.rs` — Parity tests asserting AST backends match the regex backends.

## Current Status

Each regex backend has compiled patterns; AST backends exist for TypeScript, Python, Rust, C, C++, Scala, Erlang, Elixir, Perl, and Lisp/Scheme/Emacs Lisp under `src/exports/ast/` (Nim and Crystal remain regex-only — no published tree-sitter grammar crate for either). The optional real-world Swift verification remains portable when its external fixture paths are absent and warning-free under current stable Clippy.

## Notes

- This is the only multi-file module in the project (~14 source files under `src/exports/`, plus the `ast/` subdirectory).
- Language detection is purely extension-based — no shebang or content sniffing.
- The validator uses this module bidirectionally: undocumented exports = warning, documented-but-missing exports = error.
