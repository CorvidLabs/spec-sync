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
- **Static coverage is non-vacuous**: HTML, HTM, and CSS files participate in default source discovery even though they expose no API symbols.
- **Generated companion markers fail strict**: Every known artifact-specific scaffold prompt emitted by the built-in templates, including all Layout, Components, Tokens, and Assets design bullets, emits a path-and-line warning outside fenced examples; strict mode promotes those warnings to errors.
- **Coverage gates fail inconclusively on malformed manifests**: `compute_coverage_checked` propagates
  malformed, unreadable, unsupported, or unconfined Gradle errors to CLI and MCP gate callers.
  Raw drive-qualified module identities, interpolated/encoded paths, unsafe recognized Gradle
  manifests, unsupported/dynamic project-directory methods, and symlink/reparse components in
  derived directories therefore cannot become partial or outside coverage. The original
  `compute_coverage` API remains as a compatibility wrapper and produces a zero-percent report
  carrying an inconclusive diagnostic.
- **Shared exact-byte validation core**: `validate_spec_content` accepts pre-read spec bytes and
  never opens the logical `spec_path` or adjacent companions. The path still anchors diagnostics
  and mapped-source checks, which retain normal path-based behavior.
  `validate_spec_content_with_sources` is the crate-private stronger seam: callers provide a
  `SourceSnapshot` map, and validation performs mapped-source checks and supplied-content export
  extraction without ambient source-path access. `validate_spec` preserves existing callers,
  including companion checks, by reading once and delegating to the shared core.

## Files to Read First

- `src/validator.rs` — Core validation engine: spec validation, file/LOC coverage, module detection, and cross-project reference handling.
- `src/parser.rs` — Used heavily by the validator for frontmatter and symbol extraction.
- `src/exports/mod.rs` — Used for API surface validation (comparing spec symbols to code exports).

## Current Status

CHG-0063 implementation is present. The validator powers `specsync check`, coverage, and MCP;
checked discovery prevents malformed or unconfined Gradle settings from producing partial
coverage, while
`validate_spec_content_with_sources` lets confined callers validate exact spec-and-source
snapshots without reopening paths.
Fresh CHG definition reapproval, focused final-tree reruns, independent reviews, hosted-Windows
runtime, repository/CI, trust, and Attest evidence remain pending. Its in-file
regression-test module intentionally precedes coverage helpers, so the narrow
`items_after_test_module` Clippy allowance stays localized.

## Notes

- SQL schema table extraction (`get_schema_table_names()`) supports `CREATE TABLE` statements for validating `db_tables` frontmatter fields. When `schema_columns` are supplied, `validate_spec` also cross-checks documented `### Schema:` column tables against migrations: a documented column absent from migrations is an ERROR, a migration column absent from the spec is a WARNING, and a type mismatch is a WARNING.
- Status gates validation depth: `archived` specs skip all checks; `draft` specs check structure only; `review` specs check sections but skip Public API and API-surface validation; `active`/`stable`/`deprecated` get the full pass.
- Non-draft, non-review specs warn when inline `## Requirements`/`## Acceptance Criteria` appear in the technical spec. A companion `requirements.md` is validated when present but is no longer mandatory for every module.
- `validate_spec_content` skips ambient companion reads but retains ordinary mapped-source path
  behavior; only `validate_spec_content_with_sources` treats supplied source observations as
  authoritative and forbids ambient mapped-source reopening.
- Exclude patterns use a simplified glob syntax: `**/dir/**` for directory exclusion, `**/*.ext` for extension exclusion.
