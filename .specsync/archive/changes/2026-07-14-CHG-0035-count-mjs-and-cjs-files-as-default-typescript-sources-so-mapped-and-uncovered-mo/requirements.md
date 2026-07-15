---
change: CHG-0035-count-mjs-and-cjs-files-as-default-typescript-sources-so-mapped-and-uncovered-mo
artifact: requirements
---

# Requirements

## REQ-CHG-0035-001 — Module JavaScript classification

SpecSync SHALL classify `mjs` and `cjs` filename extensions as TypeScript-family source files in both direct extension lookup and the default TypeScript extension registry.

Acceptance criteria:

- `Language::from_extension("mjs")` and `Language::from_extension("cjs")` return `Language::TypeScript`.
- `Language::TypeScript.extensions()` contains `mjs` and `cjs` without removing existing suffixes.

## REQ-CHG-0035-002 — Non-vacuous module-file coverage

Default source discovery SHALL include `.mjs` and `.cjs` files in file and LOC coverage denominators.

Acceptance criteria:

- A mapped fixture containing `.ts`, `.css`, `.mjs`, and `.cjs` sources reports the exact file and LOC totals from all sources.
- Otherwise covered fixtures with an unmapped `.mjs` or `.cjs` file each fail strict `--require-coverage 100` and report the uncovered file in the denominator.
- Explicit `source_extensions` behavior remains unchanged.
