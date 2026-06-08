---
spec: exports.spec.md
---

## User Stories

- As a developer, I want spec-sync to extract public exports from my source files automatically so that I can validate my specs against actual code
- As a TypeScript developer, I want all export forms recognized (named, default, re-exports, wildcard) so that nothing is missed
- As a Rust developer, I want `pub` items extracted including `pub(crate)` visibility so that my module's public API is accurately captured
- As a Python developer, I want `__all__` respected when present, with fallback to top-level definitions, so that my intended public API is what gets checked
- As a Go developer, I want uppercase identifiers recognized as exports so that Go's visibility convention is supported
- As a polyglot team, I want export extraction for all 12 supported languages so that spec-sync works across our entire codebase
- As a developer, I want test files automatically excluded from export extraction so that test helpers don't pollute the public API
- As a power user, I want an opt-in AST parse mode (`ParseMode::Ast`) for TypeScript, Python, and Rust so that I get higher-fidelity extraction with regex fallback when AST parsing fails

## Acceptance Criteria

- Supports 12 languages: TypeScript/JS, Python, Rust, Go, Java, Kotlin, Swift, Dart, C#, PHP, Ruby, YAML
- Language detection is purely extension-based via `Language::from_extension` (no content sniffing)
- Symbols are deduplicated while preserving declaration order
- Unreadable files or unknown extensions return empty vector (no errors)
- TypeScript wildcard re-exports (`export * from`) are followed one level deep via `resolve_ts_import` file resolver
- Ruby visibility tracking correctly handles public/private/protected toggles
- PHP skips magic methods (`__construct`, `__toString`, etc.) and private members
- `ExportLevel::Type` filters (via `filter_type_level_exports`) to only class/struct/enum declarations; `Member` includes all public symbols
- `ParseMode::Ast` uses tree-sitter for TypeScript, Python, and Rust; falls back to regex for other languages or on empty/failed AST results
- Test file detection uses language-specific patterns (`.test.ts`, `_test.go`, `test_*.py`, etc.) plus well-known test directory names (`tests`, `__tests__`, `fixtures`, `mocks`, ...)
- All regex patterns are compiled once via `LazyLock` for performance

## Constraints

- Regex-based parsing is the default; AST parsing (tree-sitter) is opt-in via `ParseMode::Ast` and limited to TS/Python/Rust
- Must handle malformed or partial source files gracefully (best-effort extraction; never panic)
- Each language backend lives in its own file under `src/exports/` for maintainability
- Must strip comments before extracting exports to avoid false positives
- AST results that come back empty fall back to the regex backend automatically

## Out of Scope

- AST/semantic analysis for languages other than TypeScript, Python, and Rust
- Extracting function signatures, parameter types, or return types
- Cross-file dependency resolution (except TypeScript wildcard re-exports, one level deep)
- Extracting private/internal symbols for any purpose
