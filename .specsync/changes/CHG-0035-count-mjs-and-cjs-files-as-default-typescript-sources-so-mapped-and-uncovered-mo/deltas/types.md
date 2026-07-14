## ADDED

### REQUIREMENT REQ-types-002

The shared language registry SHALL classify `.mjs` and `.cjs` as TypeScript-family sources for both direct detection and default extension discovery.

Acceptance Criteria

- Direct extension lookup maps `mjs` and `cjs` to TypeScript.
- The TypeScript default extension list includes `mjs` and `cjs` alongside existing JavaScript and TypeScript suffixes.
- Explicit source-extension filtering remains unchanged.
