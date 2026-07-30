---
spec: validator.spec.md
---

## User Stories

- As a developer, I want spec-sync to tell me when my spec references a source file that doesn't exist so that I can fix stale file references
- As a developer, I want to be warned when my code exports a symbol that isn't documented in the spec so that I remember to update documentation
- As a developer, I want an error when my spec documents a symbol that doesn't exist in the code so that phantom API entries are caught
- As a developer, I want file and line-of-code coverage metrics so that I can measure how much of my codebase is documented
- As a team lead, I want cross-project dependency references (`owner/repo@module`) to be recognized and skipped during local validation so that multi-repo setups don't produce false errors
- As a developer, I want Levenshtein-distance suggestions when a source file isn't found so that typos in file paths are easy to fix
- As a developer, I want schema table/column validation against my SQL migrations so that database documentation stays in sync with actual schema

## Acceptance Criteria

- Bidirectional validation: spec documents non-existent export = ERROR; code exports undocumented symbol = WARNING
- Missing frontmatter fields (module, version, status) produce errors, not warnings
- Cross-project refs (`owner/repo@module` format) are detected and skipped during local validation
- Coverage computation excludes test files and configured exclude patterns
- `find_spec_files` returns results sorted by path
- Schema validation uses configurable regex pattern via `schema_pattern` config
- File path suggestions use Levenshtein distance with max distance of 3
- Flat source files (not in subdirectories) are detected as modules, excluding common entry points (main.rs, lib.rs, mod.rs, index.ts, etc.)
- Source discovery respects `source_extensions` config
- Requirements companions are validated when present but remain optional for technical/internal modules under adaptive artifact policy.
- `validate_spec_content` applies the normal single-spec validation contract to caller-provided
  bytes without opening `spec_path` or adjacent companions; the path remains diagnostic/source
  context, while mapped sources retain normal path-based behavior.
- `validate_spec_content_with_sources` accepts a capability-confined `SourceSnapshot` map and
  validates mapped sources without reopening their ambient paths.
- `validate_spec` reads the spec once and delegates its exact bytes to the shared content validator.

## Constraints

- Validation must be fast enough for watch mode (~500ms debounce between runs)
- Must accumulate all errors before reporting (not fail-fast on first error)
- Error messages must include file paths and specific symbol/section names for actionability
- Pre-read content validation must not reopen the logical spec path for spec bytes.
- Supplied-source validation must treat its `SourceSnapshot` map as authoritative and must not
  reopen mapped source paths.

## Out of Scope

- Auto-fixing validation errors (that's the `--fix` flag in CLI, which only handles undocumented exports)
- Validating spec prose quality or completeness (that's the scoring module)
- Type-checking or semantic validation of source code
- Validating that spec behavioral examples are accurate

### REQ-validator-001

The validator SHALL enforce bidirectional code-contract, metadata, dependency, schema, and coverage
rules while accumulating actionable findings, and SHALL support exact pre-read spec snapshots
without reopening their logical paths.

Acceptance Criteria

- Bidirectional validation reports a documented-but-missing export as an error and an undocumented
  code export as a warning.
- Missing required frontmatter fields (`module`, `version`, `status`, `files`) are errors.
- Cross-project references are recognized and skipped during local validation.
- Coverage excludes test files and configured exclude patterns.
- `find_spec_files` returns sorted results.
- Schema validation uses the configured `schema_pattern`.
- Missing source suggestions use Levenshtein distance with a maximum distance of three.
- Flat source files are detected as modules while common entry points are excluded.
- Source discovery respects configured `source_extensions`.
- Requirements companions are validated when present and remain optional for technical/internal
  modules under adaptive artifact policy.
- `validate_spec_content` applies normal single-spec validation to caller-provided spec bytes.
- `spec_path` remains the logical location for diagnostics and mapped-source resolution, but is not
  reopened to obtain spec content; adjacent companion reads are deliberately skipped for the
  pre-read spec-content API, while mapped sources retain normal path-based behavior.
- CRLF normalization and spec-size policy are computed from the supplied content.
- `validate_spec` preserves path-based compatibility by reading once and delegating the exact bytes
  to `validate_spec_content`.
- `SourceSnapshot` represents `Present`, `Missing`, `Rejected`, and `Unreadable` mapped-source
  observations.
- `validate_spec_content_with_sources` validates supplied spec bytes and supplied mapped-source
  observations without reopening either through ambient project paths.
- Supplied-content export extraction uses retained source bytes and does not resolve TypeScript
  wildcard imports through ambient paths.

### REQ-validator-002

Coverage SHALL measure configured static content without presenting a vacuous successful percentage.

Acceptance Criteria

- Mapped HTML reports one covered file out of one.
- Unmapped HTML reports zero covered files out of one and fails a 100 percent gate.
- Excluded assets remain excluded and static files require no exported symbols.
- A zero-file project is reported distinctly from measured 100 percent coverage.
- Coverage cannot report a percentage from a partial retained snapshot after a link/reparse,
  special entry, identity replacement, invalid UTF-8 input, or deterministic budget failure.

### REQ-validator-003

Strict validation SHALL reject known unfilled companion scaffold markers with artifact-specific line diagnostics.

Acceptance Criteria

- Generated companion markers are recognized deterministically by artifact type.
- Concrete replacement prose passes.
- Similar prose and fenced examples are ignored.
- Diagnostics identify companion path line and required correction.

### REQ-validator-004

Strict validation SHALL discover default static projects and reject every unfilled marker emitted by built-in companion templates.

Acceptance Criteria

- Zero-config root and nested HTML, HTM, and CSS files select their containing source directory.
- Ignored directories remain excluded from static discovery.
- Every generated Layout, Components, Tokens, and Assets design marker produces an artifact-specific line diagnostic.
- Concrete replacements pass while fenced examples and similar prose remain ignored.

### REQ-validator-005

Configuration-driven source discovery SHALL include paths without a filename extension when `include_extensionless` is true and SHALL apply that rule consistently across validation and generation commands.

Acceptance Criteria

- Extensionless-only strict coverage measures one mapped file and non-zero LOC.
- Mixed strict coverage measures both extensionless and explicitly configured suffixed files with non-zero LOC.
- Coverage, generation, scaffold, new-spec, wizard, diff, and output scans share the extensionless rule.
- Wizard discovery excludes directory entries before matching module names or extension rules.
- Omitted or false configuration preserves existing source selection.

### REQ-validator-006

Default source discovery SHALL include `.mjs` and `.cjs` files in strict file and LOC coverage denominators.

Acceptance Criteria

- Mapped module files increase measured file and LOC totals using their real contents.
- An uncovered `.mjs` or `.cjs` file prevents strict 100 percent coverage from passing.
- Coverage output reports non-vacuous exact totals for mixed default-language projects.

### REQ-validator-007

Validation SHALL treat safe normalized missing draft file mappings as planned by default without adding nonexistent files to current coverage.

Acceptance Criteria

- Draft planned mappings pass strict validation with explicit notices.
- Activating the spec or enabling `require_draft_files` restores the missing-file error.
- Creating the file transitions it to normal mapping and coverage.
- Existing files retain containment, readability, and duplicate-ownership validation; archived specs never contribute owners.
- Incremental checks detect owners from unchanged cached specs.
- Redundant dot segments cannot create coverage mismatches.
- Absolute, parent-segment, prefixed, and backslash mappings remain errors in every lifecycle status and never count toward ownership or coverage.
- A missing planned leaf beneath an existing symlinked parent that resolves outside the project or cannot be resolved is rejected before notice emission.

### REQ-validator-008

Coverage gates SHALL use fallible checked manifest discovery and SHALL report malformed, unreadable,
unsupported, or unconfined Gradle discovery as inconclusive instead of accepting partial coverage
or traversing outside the retained project root.

Acceptance Criteria

- `compute_coverage_checked` propagates checked manifest-discovery errors without producing a
  partial `CoverageReport`.
- CLI and MCP coverage/enforcement callers use checked coverage and fail with an inconclusive
  diagnostic.
- Raw drive-qualified module identities, dynamic/unsupported project-directory mutators, and
  symlink/reparse components in Gradle-derived directories propagate as checked errors before
  source probing, traversal, partial totals, or generation.
- Interpolated/encoded-dynamic Gradle strings and unsafe or oversized Gradle manifest endpoints
  propagate as checked errors before partial totals, outside reads, or generation.
- After checked manifest discovery, every configured or manifest-derived source tree is traversed
  through one retained project-root capability with no-follow directory opens and non-blocking,
  identity-checked regular-file reads. Post-discovery replacement, links/reparse points, and
  special files fail every coverage gate before totals, disclosure, or generated output.
- Checked coverage acquires configured source roots and source bytes through retained no-follow
  handles, binds directory/file identity before and after traversal, and derives file, LOC,
  immediate-directory, and flat-file module results from that snapshot. Post-discovery
  symlink/junction replacement fails inconclusive for every coverage gate before outside reads.
- Caller-selected spec ownership reads, manifest discovery, spec-module enumeration, source
  traversal, and final root verification share one retained project capability. Traversal is
  sorted and iterative with 8 MiB per input file, 64 MiB cumulative bytes, 100,000 entries, and
  256 path components.
- Root retention precedes configuration and omitted-source manifest/source detection. Explicit
  source roots skip autodetection; nested configuration/manifest parents remain reachable from the
  retained root. Every selected spec and source inventory entry is charged to the shared entry
  bound, and selected-spec discovery identity remains authoritative through ownership parsing.
- Separate deterministic checkpoints immediately after root retention and after discovery scope
  checked coverage; gate callers propagate their inconclusive errors.
- Invalid UTF-8 source names/content, special entries, links/reparse points, root/directory/file
  identity replacement, and exhausted bounds fail inconclusive before partial coverage totals.
- `compute_coverage` remains available for compatibility and returns a zero-percent report carrying
  an inconclusive module diagnostic when checked discovery fails.

### REQ-validator-009

Schema-aware validation SHALL compare canonical identities from the command invocation's checked
schema snapshot and SHALL never pass vacuously when schema validation was requested.

Acceptance Criteria

- Quoted, qualified, and mixed-case table declarations compare through one canonical identity.
- An unqualified declaration may match a unique qualified table leaf; a qualified declaration
  requires the full identity.
- Invalid or captureless `schema_pattern` configuration is visible and cannot silently erase schema
  validation.
- Declared `db_tables` without a configured readable schema produce a finding instead of a vacuous
  pass.

