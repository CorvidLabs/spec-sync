---
spec: exports.spec.md
---

## Tasks

- [ ] Add support for C/C++ header exports (`.h`/`.hpp` files)
- [ ] Handle TypeScript `export =` (CommonJS-style default export)
- [ ] Handle Python conditional exports (platform-specific `__all__` modification)
- [ ] Add Rust `pub(crate)` visibility filtering — currently all `pub` items are treated as exported

## Done

- [x] Implement export extraction for all 9 languages
- [x] TypeScript: declarations, re-exports, wildcards, namespace re-exports, defaults
- [x] Python: `__all__` precedence, top-level defs fallback
- [x] Rust: pub fn/struct/enum/trait/type/const/static/mod
- [x] Go: capitalized names, methods, type declarations
- [x] Swift, Kotlin, Java, C#, Dart: public type and member extraction
- [x] Comment and string literal stripping across all backends
- [x] Two-level export granularity (Type vs Member)
- [x] Test file detection (`is_test_file()`)

## Gaps

- Regex-based parsing can miss edge cases: conditional exports, computed property names, decorator-generated exports
- No support for re-exports in languages other than TypeScript
- Dart backend doesn't distinguish `part of` visibility

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
