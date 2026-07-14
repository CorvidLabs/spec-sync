---
change: CHG-0036-support-commonjs-exports-for-newly-discovered-cjs-modules-without-changing-esm
artifact: requirements
---

# Requirements

## REQ-exports-002

The TypeScript/JavaScript export scanner SHALL report statically named CommonJS exports in both regex and AST modes without changing existing ESM results.

Acceptance Criteria

- `exports.foo = ...` and `module.exports.foo = ...` report `foo`.
- Top-level shorthand and named keys in `module.exports = { ... }` are reported.
- Comments, strings, computed keys, and statically unresolved spreads do not report false exports.
- Mixed ESM/CommonJS input is stable and deduplicated in both parsing modes.
