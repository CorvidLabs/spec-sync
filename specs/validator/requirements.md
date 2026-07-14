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

## Constraints

- Validation must be fast enough for watch mode (~500ms debounce between runs)
- Must accumulate all errors before reporting (not fail-fast on first error)
- Error messages must include file paths and specific symbol/section names for actionability

## Out of Scope

- Auto-fixing validation errors (that's the `--fix` flag in CLI, which only handles undocumented exports)
- Validating spec prose quality or completeness (that's the scoring module)
- Type-checking or semantic validation of source code
- Validating that spec behavioral examples are accurate

### REQ-validator-001

The validator SHALL enforce bidirectional code-contract, metadata, dependency, schema, and coverage rules while accumulating actionable findings.

Acceptance Criteria
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

### REQ-validator-002

Coverage SHALL measure configured static content without presenting a vacuous successful percentage.

Acceptance Criteria

- Mapped HTML reports one covered file out of one.
- Unmapped HTML reports zero covered files out of one and fails a 100 percent gate.
- Excluded assets remain excluded and static files require no exported symbols.
- A zero-file project is reported distinctly from measured 100 percent coverage.

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

Validation SHALL treat safe missing draft file mappings as planned by default without adding nonexistent files to current coverage.

Acceptance Criteria

- Draft planned mappings pass strict validation with explicit notices.
- Activating the spec or enabling `require_draft_files` restores the missing-file error.
- Creating the file transitions it to normal mapping and coverage.
- Existing files retain containment, readability, and duplicate-ownership validation.
- Unsafe paths remain errors in every lifecycle status.
