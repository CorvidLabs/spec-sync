---
change: CHG-0038-harden-commonjs-export-extraction-and-exclude-module-javascript-tests-from-gener
artifact: testing
---

# Testing

- `REQ-exports-004`: focused TypeScript scanner tests cover chained assignments,
  function-local aliases, regex literals, and CommonJS classes at type level.
- `REQ-cmd-new-002`: a command fixture proves `new` omits `.test.cjs` and
  `.spec.mjs` while retaining production sources.
- `REQ-cmd-scaffold-002`: a scaffold fixture proves the same shared exclusion.

Closing evidence requires all unit and integration tests, the complete Fledge verify
lane, strict 100 percent file and LOC coverage, every hosted matrix, and Trust.
