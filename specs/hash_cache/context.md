---
spec: hash_cache.spec.md
---

## Key Decisions

- **Conservative on uncertainty**: a missing or corrupt `.specsync/hashes.json`, and any unreadable file, are all treated as "changed". The cache can only ever cause *more* validation, never skip a file that genuinely changed.
- **SHA-256 streamed in 8KB chunks**: `hash_file` reads incrementally so large source files don't blow memory; the digest is stored as a hex string.
- **Normalized path keys**: `normalize_rel` rewrites separators to `/` so the same cache works across macOS/Linux/Windows checkouts.
- **Companion classification split**: `requirements.md` is its own `ChangeKind::Requirements`; `context.md`/`tasks.md`/`testing.md`/`design.md` collapse into `ChangeKind::Companion`. Both plain and legacy `{module}.<suffix>` names are probed by `find_companion_files`.
- **Cheap frontmatter scan**: `extract_frontmatter_files` string-matches the `files:` block instead of invoking the full YAML parser, keeping the change-detection path fast.
- **Prune on update**: `update_cache` re-hashes everything relevant and then drops entries for files that no longer exist, bounding cache size.
- **Snapshots are the previous findings**: hashes decide *whether* to skip re-validation; `snapshots` hold the result that must still be reported. A hash hit without a replayable snapshot is a miss.

## Files to Read First

- `src/hash_cache.rs` — entire module: `HashCache` (load/save/hash/is_changed/prune), `classify_changes`, `find_companion_files`, `update_cache`, `extract_frontmatter_files`.

## Current Status

Stable and complete. Used by `cmd_check` for incremental validation. Public API spans the `HashCache` struct, `ChangeKind`/`ChangeClassification`, and the classify/filter/update free functions.

## Notes

- Depends on `sha2` (hashing) and `serde`/`serde_json` (cache serialization); conceptually pairs with `parser` for frontmatter.
- `ChangeClassification::has` takes `&ChangeKind`; `is_changed(&self)` reports whether any change was recorded.
- Cache lives under `.specsync/` alongside other spec-sync state.
