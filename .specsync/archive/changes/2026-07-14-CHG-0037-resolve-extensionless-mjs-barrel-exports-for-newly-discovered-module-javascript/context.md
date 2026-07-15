---
change: CHG-0037-resolve-extensionless-mjs-barrel-exports-for-newly-discovered-module-javascript
artifact: context
---

# Context

SpecSync 5 now discovers `.mjs` and `.cjs` as TypeScript-family source files, and
CHG-0036 teaches both export backends to extract static CommonJS names. Relative
export-star resolution still probes only TypeScript, JavaScript, MTS, and CTS file
variants. An extensionless barrel such as `export * from "./values"` therefore
misses `values.mjs`, making strict validation report false removed or undocumented
exports.

The fix stays inside the existing one-level relative resolver. It adds module
JavaScript file and index variants while preserving the current probe order,
non-relative import rejection, unreadable-file behavior, and regex/AST fallback.
