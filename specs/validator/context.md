---
spec: validator.spec.md
---

## Key Decisions

- **Bidirectional validation**: Spec documents non-existent export = ERROR (spec is wrong). Code exports undocumented symbol = WARNING (spec is incomplete). This asymmetry reflects that incorrect docs are worse than incomplete docs.
- **Missing frontmatter fields are errors**: `module`, `version`, `status`, and `files` are all required. Missing any of these is an error, not a warning, because downstream modules depend on them.
- **Cross-project refs skipped locally**: References in `owner/repo@module` format are silently skipped during `specsync check`. They're only validated with `specsync resolve --remote`.
- **Levenshtein suggestions**: When a referenced file doesn't exist, the validator suggests similar filenames (edit distance ≤ 3) to help catch typos.
- **Coverage excludes tests**: Test files (detected by `is_test_file()`) are excluded from coverage metrics, since test code doesn't need specs.
- **Module detection cascade**: User-defined modules (config) → manifest-discovered modules → subdirectory scanning → flat file detection. Each level is a fallback.

## Files to Read First

- `src/validator.rs` — Core validation engine: spec validation, file/LOC coverage, module detection, and cross-project reference handling.
- `src/parser.rs` — Used heavily by the validator for frontmatter and symbol extraction.
- `src/exports/mod.rs` — Used for API surface validation (comparing spec symbols to code exports).

## Current Status

Fully implemented. The validator is the heart of spec-sync — it powers `specsync check`, `specsync coverage`, and is exposed via MCP. Its in-file regression-test module intentionally precedes coverage helpers, so the narrow `items_after_test_module` Clippy allowance is localized to that test module rather than weakening project-wide lint policy.

## Notes

- SQL schema table extraction (`get_schema_table_names()`) supports `CREATE TABLE` statements for validating `db_tables` frontmatter fields. When `schema_columns` are supplied, `validate_spec` also cross-checks documented `### Schema:` column tables against migrations: a documented column absent from migrations is an ERROR, a migration column absent from the spec is a WARNING, and a type mismatch is a WARNING.
- Status gates validation depth: `archived` specs skip all checks; `draft` specs check structure only; `review` specs check sections but skip Public API and API-surface validation; `active`/`stable`/`deprecated` get the full pass.
- Non-draft, non-review specs also warn when inline `## Requirements`/`## Acceptance Criteria` appear in the spec body or when the companion `requirements.md` is missing.
- Exclude patterns use a simplified glob syntax: `**/dir/**` for directory exclusion, `**/*.ext` for extension exclusion.
