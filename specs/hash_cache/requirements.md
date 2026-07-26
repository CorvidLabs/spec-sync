---
spec: hash_cache.spec.md
---

## User Stories

- As a developer on a large spec set, I want `specsync check` to re-validate only the specs that actually changed so that validation stays fast
- As a developer, I want changes to a spec's backing source files (`files:` in frontmatter) to also trigger re-validation so that drift between code and spec is caught
- As a developer, I want companion file edits (requirements/context/tasks/testing/design) to mark the parent spec as changed so that companion-aware checks re-run
- As a CI operator, I want the cache to be self-healing on first run or corruption so that a missing/invalid cache never blocks validation — it just re-checks everything
- As a maintainer, I want stale cache entries for deleted files pruned automatically so that the cache file does not grow without bound
- As a CI integrator, I want warm-cache findings to be identical to cold validation so repeated runs never erase warnings or report zero checked specs
- As a maintainer, I want malformed, incompatible, stale, or internally inconsistent cache snapshots rejected automatically so cache state can never create a false green

## Acceptance Criteria

- Cache persists to `{root}/.specsync/hashes.json` (pretty JSON); the `.specsync/` directory is created on `save`
- `HashCache::load` returns an empty cache when the file is missing or unparseable (treating all files as changed)
- `hash_file` computes SHA-256 in 8KB chunks and returns a hex digest, or `None` if the file cannot be read
- `is_changed` returns true when a path is new or its current hash differs from the cached value; unreadable files are treated as changed
- Path keys are normalized to forward slashes (`normalize_rel`) for cross-platform consistency
- Hash and snapshot maps serialize in stable path order
- `classify_changes` reports `ChangeKind::Spec`, `Requirements`, `Companion`, and/or `Source` for one spec; the requirements companion maps to `Requirements`, while context/tasks/testing/design map to `Companion`
- Companion detection covers both the plain convention (`requirements.md`, `context.md`, `tasks.md`, `testing.md`, `design.md`) and legacy `{module}.<suffix>` prefixed names
- `classify_all_changes` / `filter_unchanged` return only specs that have detected changes
- `update_cache` re-hashes every spec, its companions, and its source files after validation, then prunes deleted entries
- `extract_frontmatter_files` extracts the `files:` list with lightweight string matching, not the full YAML parser
- Cache and per-spec snapshot schemas are explicitly versioned; legacy, malformed, unknown-version, and unknown-shape state forces validation
- Each cached snapshot contains the cold-run display path, complete errors, filtered warnings, and notices plus an integrity digest and an input digest
- Snapshot input binding covers the spec, companions, frontmatter source paths, current SpecSync package version, config, recursive schema files, `.specsyncignore`, and the sorted complete spec inventory
- A snapshot with changed diagnostics, changed inputs, missing data, or mismatched version is not replayable
- Snapshot recording succeeds only when the current post-validation input digest matches the digest captured immediately before validation

## Constraints

- Hashing must stream in chunks (8KB) to avoid loading large files fully into memory
- Cache corruption or absence must never error — fall back to "everything changed"
- Cache validation must fail closed: uncertainty causes re-validation, never an empty successful replay
- Path normalization is mandatory so cache keys are stable across OSes
- Companion-name detection must support both current and legacy naming without false positives
- `extract_frontmatter_files` must not pull in the full parser (kept cheap for the hot path)

## Out of Scope

- Hashing strategies other than SHA-256
- Tracking changes to files not referenced by a spec (spec, its companions, and its `files:` only)
- Cross-run change history or diffing (only last-known hash is stored)
- Watching the filesystem for live changes (see the `watch` module)
- Validating spec content (only change detection lives here)

### REQ-hash-cache-001

The hash cache SHALL classify spec, requirement, companion, source, and global-input drift deterministically and persist only safe local state.

Acceptance Criteria
- Cache persists to `{root}/.specsync/hashes.json` (pretty JSON); the `.specsync/` directory is created on `save`
- `HashCache::load` returns an empty cache when the file is missing or unparseable (treating all files as changed)
- `hash_file` computes SHA-256 in 8KB chunks and returns a hex digest, or `None` if the file cannot be read
- `is_changed` returns true when a path is new or its current hash differs from the cached value; unreadable files are treated as changed
- Path keys are normalized to forward slashes (`normalize_rel`) for cross-platform consistency
- `classify_changes` reports `ChangeKind::Spec`, `Requirements`, `Companion`, and/or `Source` for one spec; the requirements companion maps to `Requirements`, while context/tasks/testing/design map to `Companion`
- Companion detection covers both the plain convention (`requirements.md`, `context.md`, `tasks.md`, `testing.md`, `design.md`) and legacy `{module}.<suffix>` prefixed names
- `classify_all_changes` / `filter_unchanged` return only specs that have detected changes
- `update_cache` re-hashes every spec, its companions, and its source files after validation, then prunes deleted entries
- `extract_frontmatter_files` extracts the `files:` list with lightweight string matching, not the full YAML parser

### REQ-hash-cache-002

The validation cache SHALL persist complete versioned per-spec snapshots and replay them only when their integrity and exact validation inputs remain current.

Acceptance Criteria
- Warm snapshots preserve errors, filtered warnings, and notices in deterministic order.
- Format or snapshot version mismatch, malformed shape, missing snapshot, changed diagnostic integrity, or changed input digest forces re-validation.
- Input binding includes spec, companion, source, config, recursive schema, ignore-rule, tool-version, and complete spec-inventory state.
- Local hash-map edits cannot make a changed source replay an older snapshot because replay independently hashes the current bound inputs.
- Inputs changing during validation prevent snapshot publication, so diagnostics can never be bound to bytes they did not validate.
