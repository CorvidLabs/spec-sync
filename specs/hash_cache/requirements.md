---
spec: hash_cache.spec.md
---

## User Stories

- As a developer on a large spec set, I want `specsync check` to re-validate only the specs that actually changed so that validation stays fast
- As a developer, I want changes to a spec's backing source files (`files:` in frontmatter) to also trigger re-validation so that drift between code and spec is caught
- As a developer, I want companion file edits (requirements/context/tasks/testing/design) to mark the parent spec as changed so that companion-aware checks re-run
- As a CI operator, I want the cache to be self-healing on first run or corruption so that a missing/invalid cache never blocks validation — it just re-checks everything
- As a maintainer, I want stale cache entries for deleted files pruned automatically so that the cache file does not grow without bound

## Acceptance Criteria

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

## Constraints

- Hashing must stream in chunks (8KB) to avoid loading large files fully into memory
- Cache corruption or absence must never error — fall back to "everything changed"
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

The `hash_cache` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change, including error reporting and coverage/enforcement edges that those fixes address.

Acceptance Criteria
- Related `cargo test` coverage for `hash_cache` remains green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

