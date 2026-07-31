---
spec: exports.spec.md
---

## User Stories

- As a developer, I want spec-sync to extract public exports from my source files automatically so that I can validate my specs against actual code
- As a TypeScript developer, I want all export forms recognized (named, default, re-exports, wildcard) so that nothing is missed
- As a Rust developer, I want plain `pub` and crate-visible `pub(crate)` items extracted from every listed file so a module spec captures both external API and crate collaboration contracts
- As a Python developer, I want `__all__` respected when present, with fallback to top-level definitions, so that my intended public API is what gets checked
- As a Go developer, I want uppercase identifiers recognized as exports so that Go's visibility convention is supported
- As a polyglot team, I want export extraction for all 33 supported languages so that spec-sync works across our entire codebase
- As a developer, I want test files automatically excluded from export extraction so that test helpers don't pollute the public API
- As a power user, I want an opt-in AST parse mode (`ParseMode::Ast`) for TypeScript, Python, Rust, C, C++, Scala, Erlang, Elixir, Perl, and Lisp/Scheme/Emacs Lisp so that I get higher-fidelity extraction with regex fallback when AST parsing fails
- As a capability-confined validator, I want export extraction from retained source text without
  ambient path resolution so that path replacement or wildcard imports cannot redirect validation

## Acceptance Criteria

- Supports 33 languages: TypeScript/JS, Python, Rust, Go, Java, Kotlin, Swift, Dart, C#, PHP, Ruby, YAML, C, C++, Scala, Crystal, Nim, Erlang, Elixir, Perl, Lisp (Common Lisp/Scheme/Emacs Lisp), Haskell, Lua, R, OCaml, Groovy, F#, Clojure, D, Objective-C, Bash, PowerShell, Vala
- Language detection is purely extension-based via `Language::from_extension` (no content sniffing)
- Symbols are deduplicated while preserving declaration order
- Unreadable files or unknown extensions return empty vector (no errors)
- TypeScript wildcard re-exports (`export * from`) are followed one level deep via `resolve_ts_import` file resolver
- Ruby visibility tracking correctly handles public/private/protected toggles
- PHP skips magic methods (`__construct`, `__toString`, etc.) and private members
- `ExportLevel::Type` filters (via `filter_type_level_exports`) to only class/struct/enum declarations; `Member` includes all public symbols
- `ParseMode::Ast` uses tree-sitter for TypeScript, Python, Rust, C, C++, Scala, Erlang, Elixir, Perl, and Lisp/Scheme/Emacs Lisp; falls back to regex for other languages (Nim, Crystal have no published tree-sitter grammar) or on empty/failed AST results
- Test file detection uses language-specific patterns (`.test.ts`, `_test.go`, `test_*.py`, etc.) plus well-known test directory names (`tests`, `__tests__`, `fixtures`, `mocks`, ...)
- All regex patterns are compiled once via `LazyLock` for performance
- Rust regex and AST modes include `pub` and `pub(crate)` declarations and re-exports across all listed files while excluding `pub(super)`, `pub(self)`, and `pub(in ...)`
- `get_exported_symbols_from_content` parses caller-supplied source text without reopening its
  logical path and does not resolve TypeScript wildcard imports through ambient filesystem paths

## Constraints

- Regex-based parsing is the default; AST parsing (tree-sitter) is opt-in via `ParseMode::Ast` and limited to TS/Python/Rust/C/C++/Scala/Erlang/Elixir/Perl/Lisp
- Must handle malformed or partial source files gracefully (best-effort extraction; never panic)
- Each language backend lives in its own file under `src/exports/` for maintainability
- Must strip comments before extracting exports to avoid false positives
- AST results that come back empty fall back to the regex backend automatically
- Supplied-content extraction must not derive additional filesystem authority from `file_path`

## Out of Scope

- AST/semantic analysis for languages other than TypeScript, Python, Rust, C, C++, Scala, Erlang, Elixir, Perl, and Lisp/Scheme/Emacs Lisp (notably Nim and Crystal, which have no published tree-sitter grammar crate)
- Extracting function signatures, parameter types, or return types
- Cross-file dependency resolution (except TypeScript wildcard re-exports, one level deep)
- Extracting private/internal symbols for any purpose

### REQ-exports-001

The Rust export scanner SHALL preserve every documented contract symbol across every source file listed by a spec.

Acceptance Criteria
- Regex and AST parsing include plain `pub` and crate-visible `pub(crate)` declarations, including valid whitespace variants.
- Crate-visible items and re-exports inside private inline modules are included consistently in both parse modes.
- Narrower `pub(super)`, `pub(self)`, and `pub(in ...)` declarations remain excluded.
- A multi-file fixture matching issue #334 passes strict phantom/undocumented export validation in both parse modes.

### REQ-exports-002

The TypeScript/JavaScript export scanner SHALL report statically named CommonJS exports in both regex and AST modes without changing existing ESM results.

Acceptance Criteria

- `exports.foo = ...` and `module.exports.foo = ...` report `foo`.
- Top-level shorthand and named keys in `module.exports = { ... }` are reported.
- Comments, strings, computed keys, and statically unresolved spreads do not report false exports.
- Mixed ESM/CommonJS input is stable and deduplicated in both parsing modes.

### REQ-exports-003

The TypeScript/JavaScript relative export resolver SHALL resolve extensionless
module JavaScript file and directory-index targets without changing its existing
one-level relative traversal boundary.

Acceptance Criteria

- An extensionless export-star target resolves sibling `.mjs` and `.cjs` files.
- Directory targets resolve `index.mjs` and `index.cjs` alongside existing index variants.
- Regex and AST parse modes return the same ordered, deduplicated public names.
- Non-relative, unresolved, and unreadable targets retain their existing best-effort behavior.

### REQ-exports-004

The static CommonJS scanner SHALL preserve every statically named module export
without reporting assignment-like text from non-module scopes or literals.

Acceptance Criteria

- Chained property assignments report each exported name in source order.
- Function-like local `exports` or `module` aliases do not create module exports.
- Regular-expression literals cannot create property exports or corrupt object scanning.
- Type-level scans preserve local and inline classes exported through CommonJS.
- Regex and AST modes remain ordered, deduplicated, and compatible with ESM.

### REQ-exports-005

Snapshot export extraction SHALL parse caller-supplied source content without reopening logical
paths or resolving TypeScript wildcard imports through ambient filesystem authority.

Acceptance Criteria

- The module-internal `get_exported_symbols_from_content` entry point accepts logical path,
  caller-supplied UTF-8 text, export level, and parse mode.
- The logical path selects the language and type-filter context but is never opened.
- Regex and AST TypeScript supplied-content paths pass no wildcard resolver, so ambient sibling or
  index files cannot contribute symbols.
- Local exports present in the supplied text retain normal ordering, deduplication, parse-mode, and
  export-level behavior.

### REQ-exports-006

TypeScript and Erlang export extraction SHALL recognize their supported public declaration forms
consistently in regex and AST modes without introducing phantom symbols.

Acceptance Criteria

- TypeScript recognizes supported declaration, default, named, alias, and relative re-export forms
  with stable ordered deduplication.
- Erlang recognizes exported functions and types with arity and attribute formatting variants.
- Comments, strings, private/local declarations, unresolved imports, and malformed partial input do
  not create phantom exports.
- Regex and AST fixtures agree where both modes support the construct and retain documented
  best-effort fallback.

