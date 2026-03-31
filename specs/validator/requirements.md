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

## Constraints

- Validation must be fast enough for watch mode (~500ms debounce between runs)
- Must accumulate all errors before reporting (not fail-fast on first error)
- Error messages must include file paths and specific symbol/section names for actionability

## Out of Scope

- Auto-fixing validation errors (that's the `--fix` flag in CLI, which only handles undocumented exports)
- Validating spec prose quality or completeness (that's the scoring module)
- Type-checking or semantic validation of source code
- Validating that spec behavioral examples are accurate
