---
spec: exports.spec.md
---

## Tasks

- [ ] Add support for C/C++ header exports (`.h`/`.hpp` files)
- [x] Handle TypeScript `export =` (CommonJS-style default export) — captured in both regex and AST backends
- [ ] Add AST backends for Go, Java, Swift, Kotlin, and the remaining regex-only languages (currently TS/Python/Rust/C/C++/Scala/Erlang/Elixir/Perl/Lisp have AST backends)
- [x] Add Rust `pub(crate)` visibility filtering — `pub(crate)`, `pub(super)`, `pub(self)`, and `pub(in path)` are now excluded in both regex and AST backends

## Done

- [x] Support Android/Kotlin Multiplatform declaration modifiers, same-line annotations, and value classes
- [x] Keep optional real-world Swift verification warning-free under current stable Clippy
- [x] Implement regex export extraction for all 33 languages (TS, Python, Rust, Go, Java, Kotlin, Swift, Dart, C#, PHP, Ruby, YAML, C, C++, Scala, Crystal, Nim, Erlang, Elixir, Perl, Lisp, Haskell, Lua, R, OCaml, Groovy, F#, Clojure, D, Objective-C, Bash, PowerShell, Vala)
- [x] Comprehensive multi-agent audit of all 21 pre-existing language backends against realistic code (the "container members don't repeat the container's visibility keyword" bug class, first found in Swift protocols) — 59 confirmed bugs found and fixed across every single language
- [x] 12 new languages implemented with language-correct visibility semantics (not a naive "look for a public keyword" assumption) and adversarially verified — 11 of 12 needed real fixes during independent verification
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
- [x] Extend AST parse mode to C, C++, Scala, Erlang, Elixir, Perl, and Lisp/Scheme/Emacs Lisp (tree-sitter runtime bumped 0.24→0.26 to support these grammars' ABI 15)
- [x] Test file detection (`is_test_file()`) by filename pattern and test directory name
- [x] TypeScript wildcard resolver (`resolve_ts_import`) trying .ts/.tsx/.js/.jsx/.mts/.cts and index files

## Gaps

- Regex-based parsing can miss edge cases: conditional exports, computed property names, decorator-generated exports
- AST mode is limited to TypeScript, Python, Rust, C, C++, Scala, Erlang, Elixir, Perl, and Lisp/Scheme/Emacs Lisp; Nim and Crystal have no published tree-sitter grammar crate and remain regex-only
- No support for re-exports in languages other than TypeScript
- Dart backend doesn't distinguish `part of` visibility

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
