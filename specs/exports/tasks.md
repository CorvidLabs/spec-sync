---
spec: exports.spec.md
---

## Tasks

- [ ] Add support for C/C++ header exports (`.h`/`.hpp` files)
- [ ] Handle TypeScript `export =` (CommonJS-style default export)
- [ ] Add AST backends for Go, Java, Swift, Kotlin, and the remaining languages (currently only TS/Python/Rust)
- [ ] Add Rust `pub(crate)` visibility filtering — currently all `pub` items are treated as exported

## Done

- [x] Implement regex export extraction for all 12 languages (TS, Python, Rust, Go, Java, Kotlin, Swift, Dart, C#, PHP, Ruby, YAML)
- [x] TypeScript: declarations, re-exports, wildcards, namespace re-exports, defaults
- [x] Python: `__all__` precedence, top-level defs fallback
- [x] Rust: pub fn/struct/enum/trait/type/const/static/mod (incl. `pub(crate)`, `pub async/unsafe`)
- [x] Go: capitalized names, methods, type declarations
- [x] Swift, Kotlin, Java, C#, Dart, PHP, Ruby: public type and member extraction
- [x] YAML backend: top-level keys, named entries under well-known parent keys, anchors
- [x] Comment and string literal stripping across all backends
- [x] Two-level export granularity (Type vs Member) via `filter_type_level_exports`
- [x] Opt-in AST parse mode (`ParseMode::Ast`) for TypeScript, Python, Rust with regex fallback
- [x] AST parity tests in `src/exports/ast/tests.rs` cross-checking AST vs regex output
- [x] Test file detection (`is_test_file()`) by filename pattern and test directory name
- [x] TypeScript wildcard resolver (`resolve_ts_import`) trying .ts/.tsx/.js/.jsx/.mts/.cts and index files

## Gaps

- Regex-based parsing can miss edge cases: conditional exports, computed property names, decorator-generated exports
- AST mode is limited to TypeScript, Python, and Rust; all other languages are regex-only
- No support for re-exports in languages other than TypeScript
- Dart backend doesn't distinguish `part of` visibility

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
