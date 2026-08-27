---
spec: exports.spec.md
---

## Key Decisions

- **Regex by default, AST opt-in**: All language backends use compiled regexes (`LazyLock<Regex>`). `ParseMode::Ast` enables tree-sitter backends for TypeScript, Python, Rust, C, C++, Scala, Erlang, Elixir, Perl, and Lisp/Scheme/Emacs Lisp; if the AST backend returns nothing (parse failure), `mod.rs` falls back to the regex backend automatically. Nim and Crystal have no published tree-sitter grammar crate, so they remain regex-only.
- **Comment stripping first**: Every backend strips string literals then comments before running export regexes, preventing false matches inside strings or commented-out code.
- **TypeScript wildcard resolution**: `export * from './foo'` is resolved one level deep via `resolve_ts_import` to prevent infinite re-export loops. Namespace re-exports (`export * as Ns`) produce the namespace name, not the inner symbols. Without a resolver, wildcard lines are skipped.
- **Static CommonJS extraction**: The TypeScript-family backend supplements ESM parsing with a shared lexical scanner for `exports.name`, `module.exports.name`, and statically named `module.exports = { ... }` keys. Both parse modes reuse it for parity; comments, literals, computed keys, and unresolved spreads remain excluded without executing source.
- **Python `__all__` precedence**: If `__all__` is defined, it is the sole source of exports. Otherwise, top-level non-underscore functions and classes are extracted.
- **Go capitalization convention**: Uppercase first letter = exported. Methods are extracted separately from functions.
- **Two export levels**: `ExportLevel::Type` (via `filter_type_level_exports`) extracts only top-level type declarations (class, struct, enum, interface); `ExportLevel::Member` extracts all public symbols. This allows specs to document at the right granularity.
- **Rust module-contract visibility**: Both scanners treat plain `pub` and `pub(crate)` as spec-visible because specs cover crate collaboration boundaries, while narrower `pub(super)`, `pub(self)`, and `pub(in ...)` paths remain excluded. Every frontmatter-listed source file participates.
- **Modern Kotlin modifier chains**: Kotlin declarations accept repeated modifiers in language-valid order rather than one hard-coded sequence. The regex backend also handles same-line annotations (including one level of nested argument parentheses), `value class`, `expect`/`actual`, and `external`, while restricted visibility still wins even when it follows another modifier. A restricted annotated type opens a non-exportable scope so its public-looking members cannot leak into the module API.
- **Supplied-content extraction is ambient-free**: The module-internal
  `get_exported_symbols_from_content` entry point receives retained UTF-8 source text and uses the
  logical file path only for language/type context. It never reopens that path and deliberately
  disables TypeScript wildcard resolution, because following a wildcard through ambient paths
  would escape the caller's snapshot capability.

## Files to Read First

- `src/exports/mod.rs` — Router, language detection, `get_exported_symbols()`/`get_exported_symbols_full()` entry points, `filter_type_level_exports`, `resolve_ts_import`, and `is_test_file()`/`is_source_file()`/`has_extension()` helpers.
- `src/exports/typescript.rs` — Most complex regex backend (re-exports, wildcards, defaults). Good reference for understanding the pattern.
- `src/exports/ast/tests.rs` — Parity tests asserting AST backends match the regex backends.

## Current Status

Each regex backend has compiled patterns; AST backends exist for TypeScript, Python, Rust, C, C++,
Scala, Erlang, Elixir, Perl, and Lisp/Scheme/Emacs Lisp under `src/exports/ast/` (Nim and Crystal
remain regex-only — no published tree-sitter grammar crate for either). TypeScript-family scanning
recognizes static ESM, TypeScript `export =`, and ordinary CommonJS names with regex/AST parity.
Retained-source validation uses the ambient-free supplied-content entry point. Rust multi-file
extraction is regression-tested in strict mode for both backends. The optional real-world Swift
verification remains portable when its external fixture paths are absent and warning-free under
current stable Clippy.

## Notes

- This is the only multi-file module in the project: 34 source files directly under `src/exports/` (33 language backends plus `mod.rs`), plus 12 more in the `ast/` subdirectory.
- Language detection is purely extension-based — no shebang or content sniffing.
- The validator uses this module bidirectionally: undocumented exports = warning, documented-but-missing exports = error.
