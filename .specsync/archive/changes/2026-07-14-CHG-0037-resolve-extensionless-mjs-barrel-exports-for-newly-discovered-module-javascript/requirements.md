---
change: CHG-0037-resolve-extensionless-mjs-barrel-exports-for-newly-discovered-module-javascript
artifact: requirements
---

# Requirements

- Preserve the existing relative-only, one-level export-star resolution boundary.
- Probe `.mjs` and `.cjs` siblings after the existing TypeScript and JavaScript variants.
- Probe `index.mjs` and `index.cjs` alongside the existing directory index variants.
- Return the same ordered, deduplicated public export set in regex and AST parse modes.
- Keep unreadable and unresolved targets best-effort and panic-free.
