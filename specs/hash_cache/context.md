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
- **Fail-closed versioning**: cache format 1 and snapshot format 1 are explicit. Missing, unknown, malformed, or unknown-shape data becomes an empty current cache so the next check validates.
- **Input-bound snapshots**: every result is bound to independently hashed spec, companion, source, config, recursive schema, ignore, tool-version, and sorted spec-inventory inputs. The stored hash map cannot by itself authorize a replay.
- **Diagnostic integrity**: each snapshot carries a digest over its version, input digest, errors, warnings, and notices. Editing a cached outcome invalidates it.
- **Race-safe publication**: the shared validator hashes inputs immediately before validation and `record_validation_snapshot` hashes them again afterward. A mismatch removes the snapshot and forces the next run to validate.

## Files to Read First

- `src/hash_cache.rs` — entire module: `HashCache` (load/save/hash/is_changed/prune), `classify_changes`, `find_companion_files`, `update_cache`, `extract_frontmatter_files`.

## Current Status

Stable and complete. Used by `cmd_check` and `rehash` for incremental validation. Public API includes `HashCache`, `CachedValidationSnapshot`, `ValidationDiagnostics`, change classification, snapshot record/replay, and classify/filter/update helpers.

## Notes

- Depends on `sha2` (hashing) and `serde`/`serde_json` (cache serialization); conceptually pairs with `parser` for frontmatter.
- `ChangeClassification::has` takes `&ChangeKind`; `is_changed(&self)` reports whether any change was recorded.
- Cache lives under `.specsync/` alongside other spec-sync state.
- `rehash` now produces replayable validation snapshots rather than hash-only state; it remains a local deterministic command.
