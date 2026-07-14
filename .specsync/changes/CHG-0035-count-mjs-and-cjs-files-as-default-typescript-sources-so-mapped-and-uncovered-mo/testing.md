---
change: CHG-0035-count-mjs-and-cjs-files-as-default-typescript-sources-so-mapped-and-uncovered-mo
artifact: testing
---

# Testing

Focused unit tests assert both direct language classification and the default TypeScript extension registry. Integration tests create real canonical specs and source files under default discovery:

- a mapped `.ts`, `.css`, `.mjs`, and `.cjs` fixture must report 4/4 files and exact non-zero LOC including both module files;
- a fully mapped `.ts` fixture plus an unmapped `.mjs` or `.cjs` source must fail strict 100 percent coverage and report 1/2 files.

The final local gate is `fledge lanes run verify`, followed by `specsync check --strict --require-coverage 100 --force` and `git diff --check`. Hosted platform, packaged consumer, security, required CI, and Trust checks remain mandatory before merge.

Focused regression commands:

- `cargo test module_javascript_extensions_are_typescript_family_sources`
- `cargo test --test integration default_discovery_counts_mjs_and_cjs_in_exact_coverage_totals`
- `cargo test --test integration uncovered_mjs_and_cjs_files_fail_strict_full_coverage`

Canonical requirement evidence:

- `REQ-types-002`: `types::language_extension_tests::module_javascript_extensions_are_typescript_family_sources` proves direct classification and default registry parity.
- `REQ-validator-006`: `check::default_discovery_counts_mjs_and_cjs_in_exact_coverage_totals` and `check::uncovered_mjs_and_cjs_files_fail_strict_full_coverage` prove exact mapped totals and strict uncovered-file failure for both suffixes.
