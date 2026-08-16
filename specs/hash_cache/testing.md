---
spec: hash_cache.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/hash_cache.rs` | cargo test hash_cache:: | `cache_round_trip`, `snapshots_round_trip_and_old_caches_revalidate`, `snapshot_integrity_and_input_changes_are_rejected`, `is_changed_detects_new_file`, `is_changed_detects_modification`, `extract_files_from_frontmatter`, `prune_removes_missing`, `classify_detects_spec_change`, `classify_detects_requirements_change`, `classify_detects_companion_change`, `classify_detects_testing_companion_change`, `classify_detects_source_change`, `companion_files_found_with_plain_names`, `update_cache_tracks_plain_companion_files` |
| `tests/integration.rs` | cargo test --test integration check_creates_hash_cache | End-to-end fixture: `check_creates_hash_cache` |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Incremental validation | 50 specs, only 3 have changed since last run | `classify_all_changes` is called | returns 3 `ChangeClassification` entries; 47 specs are skipped |
| Source file change triggers re-validation | a spec lists `src/auth.rs` in frontmatter `files:`; that file has been modified | `classify_changes` is called for the spec | returns `ChangeClassification` with `ChangeKind::Source` in changes |
| First run (no cache) | `.specsync/hashes.json` does not exist | `HashCache::load` is called | returns empty cache; all files will be classified as changed |
| Requirements change triggers staleness | `requirements.md` companion has been updated | `classify_changes` is called for the parent spec | returns `ChangeClassification` with `ChangeKind::Requirements` |
| Design or testing companion change detected | `testing.md` or `design.md` has been modified | `classify_changes` is called for the parent spec | returns `ChangeClassification` with `ChangeKind::Companion` |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Cache file missing | Returns empty cache (all files treated as changed) | Keep or add a focused assertion before changing this behavior |
| Cache file has invalid JSON | Returns empty cache silently | Keep or add a focused assertion before changing this behavior |
| File unreadable during hashing | `hash_file` returns `None`; file treated as changed | Keep or add a focused assertion before changing this behavior |
| Cannot create `.specsync/` directory | `save` returns `io::Error` | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/hash_cache.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
