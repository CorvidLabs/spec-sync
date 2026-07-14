---
change: CHG-0037-resolve-extensionless-mjs-barrel-exports-for-newly-discovered-module-javascript
artifact: docs
---

# Docs

No command-line syntax or configuration changes. The canonical exports requirement
companion records that extensionless relative export-star targets may resolve to
`.mjs`, `.cjs`, and their directory index variants. The exports context remains
the source of truth for one-level barrel traversal and parse-mode parity.
