## ADDED

### REQUIREMENT REQ-exports-005

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
