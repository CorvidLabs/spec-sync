---
spec: hash_cache.spec.md
---

## Tasks

(none open)

## Done

- [x] `HashCache` struct with `load`/`save` to `.specsync/hashes.json` (pretty JSON)
- [x] `hash_file` SHA-256 streaming in 8KB chunks
- [x] `is_changed` / `update` / `prune` per-path operations
- [x] Cross-platform path normalization (`normalize_rel`, forward slashes)
- [x] `ChangeKind` enum (Spec / Requirements / Companion / Source) and `ChangeClassification`
- [x] `classify_changes`: spec, companions, and frontmatter source files
- [x] Companion detection for plain names and legacy `{module}.<suffix>` (req/context/tasks/testing/design)
- [x] `classify_all_changes` / `filter_unchanged` returning only changed specs
- [x] `update_cache` re-hash + prune after validation
- [x] `extract_frontmatter_files` lightweight `files:` extraction
- [x] Empty-cache fallback on missing/corrupt JSON
- [x] Integration coverage: `check_creates_hash_cache`
- [x] Persist and replay per-spec validation snapshots so a warm check cannot drop findings
- [x] Populate requirements.md with user stories and acceptance criteria (2026-04-10)

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
