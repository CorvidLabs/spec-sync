---
spec: exports.spec.md
---

## User Stories

- As a developer, I want spec-sync to extract public exports from my source files automatically so that I can validate my specs against actual code
- As a TypeScript developer, I want all export forms recognized (named, default, re-exports, wildcard) so that nothing is missed
- As a Rust developer, I want `pub` items extracted including `pub(crate)` visibility so that my module's public API is accurately captured
- As a Python developer, I want `__all__` respected when present, with fallback to top-level definitions, so that my intended public API is what gets checked
- As a Go developer, I want uppercase identifiers recognized as exports so that Go's visibility convention is supported
- As a polyglot team, I want export extraction for all 11 supported languages so that spec-sync works across our entire codebase
- As a developer, I want test files automatically excluded from export extraction so that test helpers don't pollute the public API

## Acceptance Criteria

- Supports 11 languages: TypeScript/JS, Python, Rust, Go, Java, Kotlin, Swift, Dart, C#, PHP, Ruby
- Language detection is purely extension-based (no content sniffing)
- Symbols are deduplicated while preserving declaration order
- Unreadable files or unknown extensions return empty vector (no errors)
- TypeScript wildcard re-exports (`export * from`) are followed one level deep via file resolver
- Ruby visibility tracking correctly handles public/private/protected toggles
- PHP skips magic methods (`__construct`, `__toString`, etc.) and private members
- `ExportLevel::Type` filters to only class/struct/enum declarations; `Member` includes all public symbols
- Test file detection uses language-specific patterns (`.test.ts`, `_test.go`, `test_*.py`, etc.)
- All regex patterns are compiled once via `LazyLock` for performance

## Constraints

- Regex-based parsing only — no AST parsing or external language toolchains required
- Must handle malformed or partial source files gracefully (best-effort extraction)
- Each language backend must be in its own file for maintainability
- Must strip comments before extracting exports to avoid false positives

## Out of Scope

- Full AST parsing or semantic analysis
- Extracting function signatures, parameter types, or return types
- Cross-file dependency resolution (except TypeScript wildcard re-exports)
- Extracting private/internal symbols for any purpose
