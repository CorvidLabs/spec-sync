## ADDED

### REQUIREMENT REQ-exports-006

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
