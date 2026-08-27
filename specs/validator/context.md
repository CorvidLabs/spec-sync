---
spec: validator.spec.md
---

## Key Decisions

- **Retained coverage snapshot**: Checked coverage opens caller-selected spec ownership
  frontmatter, every recognized manifest, configured source roots/files, and spec-module entries
  through one retained project capability, binds directory/file/root identities across reads, and
  derives ownership/file/LOC/module results from exact bytes. Post-retention ambient path
  replacement is inconclusive before outside access.
- **Bounded iterative coverage**: Traversal is sorted and iterative, filters excluded names before
  metadata inspection, and enforces one selected-spec/source budget of 8 MiB per file, 64 MiB
  cumulative bytes, 100,000 inventory entries, and 256 path components. Special entries and invalid
  UTF-8 names/content fail closed.
- **Two checked-coverage race checkpoints**: The early checkpoint immediately follows project-root
  retention; the second follows retained manifest discovery. Gate commands propagate failures from
  both checkpoints, but the retained capability does not span unrelated command phases.
- **Retained zero-config discovery**: Configuration fallback and manifest/source autodetection run
  only when `source_dirs` is omitted. Nested configuration/manifest parents remain reachable from
  the retained root, and selected-spec inventory identities remain authoritative through ownership
  reads.
- **Bidirectional validation**: Spec documents non-existent export = ERROR (spec is wrong). Code exports undocumented symbol = WARNING (spec is incomplete). This asymmetry reflects that incorrect docs are worse than incomplete docs.
- **Missing frontmatter fields are errors**: `module`, `version`, `status`, and `files` are all required. Missing any of these is an error, not a warning, because downstream modules depend on them.
- **Cross-project refs skipped locally**: References in `owner/repo@module` format are silently skipped during `specsync check`. They're only validated with `specsync resolve --remote`.
- **Levenshtein suggestions**: When a referenced file doesn't exist, the validator suggests similar filenames (edit distance ≤ 3) to help catch typos.
- **Coverage excludes tests**: Test files (detected by `is_test_file()`) are excluded from coverage metrics, since test code doesn't need specs.
- **Module detection cascade**: User-defined modules (config) → manifest-discovered modules → subdirectory scanning → flat file detection. Each level is a fallback.
- **Static coverage is non-vacuous**: HTML, HTM, and CSS files participate in default source discovery even though they expose no API symbols.
- **Generated companion markers fail strict**: Every known artifact-specific scaffold prompt emitted by the built-in templates, including all Layout, Components, Tokens, and Assets design bullets, emits a path-and-line warning outside fenced examples; strict mode promotes those warnings to errors.
- **Coverage gates fail inconclusively on malformed manifests, but only when the manifest is what
  they were relying on**: `compute_coverage_checked` propagates malformed, unreadable, unsupported,
  or unconfined Gradle errors to CLI and MCP gate callers WHEN `source_dirs` was not stated — the
  source list would otherwise be the output of the discovery that failed. When `source_dirs` IS
  stated, discovery degrades to an empty result and the error is carried as a `manifest_notices`
  entry instead. Discovery exists to INFER what the project did not state; a failure to infer
  cannot veto what it did state. This mirrors the zero-config decision below, which `retained_config`
  already honoured and coverage did not.
  On the propagating path, raw drive-qualified module identities, interpolated/encoded paths, unsafe
  recognized Gradle manifests, unsupported/dynamic project-directory methods, and symlink/reparse
  components in derived directories therefore cannot become partial or outside coverage. On the
  degrading path they cost only manifest-declared module names: the file and LOC figures come
  entirely from the stated `config.source_dirs`, never from discovery. The original
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
coverage. Post-discovery source enumeration and reads stay bound to one retained root capability
with no-follow, non-blocking, identity-checked access, so path replacement and special entries
make all coverage gates inconclusive before totals or mutation. Spec ownership reads, manifest
discovery, spec-module enumeration, source traversal, and final root verification now share that
exact capability and bounded observation, while
`validate_spec_content_with_sources` lets confined callers validate exact spec-and-source
snapshots without reopening paths.
The latest remediation implements lazy retained autodetection, nested config/manifest-directory
reachability, selected-spec identity continuity, shared selected-spec/source accounting, and
distinct checked-coverage race checkpoints. The latest amendment preserves bounded scan fallback
after malformed manifest autodetection and retains selected source-directory identities through
checked traversal. Spec/source traversal now records sibling identities and reopens children
sequentially. Configured source roots are identity-selected without retaining all handles, then
reopened and traversed one at a time, bounding live directory handles by depth rather than sibling
or root count while preserving replacement checks. Combined results pass 45 focused validator
tests and the whole suite, which is 2,407 unit plus 407 integration tests today;
exact-tree independent review remains pending. A
command-wide immutable CLI analysis snapshot and generic structured
discovery outcomes are intentionally deferred to the later CLI/outcome/generation work outside
GitHub #414's MCP boundary. Its in-file
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

## Lesson (#723)

Fixing the parser that produced the error would have removed one rejection; fixing what an error is
ALLOWED TO DO removed the class. `compute_coverage_checked` was the only manifest-discovery caller
that propagated instead of degrading — `config.rs:66` uses `unwrap_or_else`, `validator.rs:430` falls
back to a scan — and it is the one on the path CI depends on, so an unreadable manifest made a valid
project unmeasurable however it was configured. Before propagating a discovery error, ask what the
caller was actually relying on discovery FOR: here the file and LOC figures come entirely from
`config.source_dirs`, and only module attribution came from the manifest, which is what made
degrading safe and made a notice mandatory.
