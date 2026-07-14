## ADDED

### REQUIREMENT REQ-exports-003

The TypeScript/JavaScript relative export resolver SHALL resolve extensionless
module JavaScript file and directory-index targets without changing its existing
one-level relative traversal boundary.

Acceptance Criteria

- An extensionless export-star target resolves sibling `.mjs` and `.cjs` files.
- Directory targets resolve `index.mjs` and `index.cjs` alongside existing index variants.
- Regex and AST parse modes return the same ordered, deduplicated public names.
- Non-relative, unresolved, and unreadable targets retain their existing best-effort behavior.
