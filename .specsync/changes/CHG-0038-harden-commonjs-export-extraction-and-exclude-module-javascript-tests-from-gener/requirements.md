---
change: CHG-0038-harden-commonjs-export-extraction-and-exclude-module-javascript-tests-from-gener
artifact: requirements
---

# Requirements

- Report each statically named export in a chained CommonJS assignment.
- Ignore CommonJS-looking text in regex literals and function-like local scopes.
- Preserve CommonJS class names under type-level export filtering.
- Exclude recognized test files from `new` and `scaffold` module discovery.
- Preserve ESM results, declaration order, deduplication, and best-effort parsing.
