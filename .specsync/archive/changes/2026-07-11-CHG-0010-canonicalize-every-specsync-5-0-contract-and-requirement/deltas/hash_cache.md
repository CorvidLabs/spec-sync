## ADDED

### REQUIREMENT REQ-hash-cache-001

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
