---
change: CHG-0037-resolve-extensionless-mjs-barrel-exports-for-newly-discovered-module-javascript
artifact: testing
---

# Testing

Requirement evidence:

- `REQ-exports-003`: `module_javascript_resolves_extensionless_file_and_index_barrels`
  proves sibling `.mjs`/`.cjs` and directory `index.mjs` resolution through the
  shared scanner.
- `REQ-exports-003`: `extensionless_mjs_barrel_passes_strict_in_regex_and_ast_modes`
  proves the canonical public export set passes strict validation in both parse modes.

Final validation runs all unit and integration tests, `fledge lanes run verify`,
`specsync change check`, strict 100 percent spec coverage, hosted CI matrices, and
the required Trust gate.
