---
spec: hash_cache.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/hash_cache.rs` | cargo test hash_cache:: | Existing hash/classification tests plus versioned round-trip, integrity/input rejection, and stale pre-validation digest publication refusal |
| `tests/integration.rs` | cargo test --test integration warm_cache | Warm text/JSON diagnostic replay and explicit cache counters |
| `tests/integration.rs` | cargo test --test integration cache_format_and_snapshot_version_mismatches_force_revalidation | Format/snapshot version invalidation |
| `tests/integration.rs` | cargo test --test integration tampered_snapshot_and_stale_inputs_cannot_produce_a_false_green | Snapshot integrity and independent current-input binding |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Incremental validation | 50 specs, only 3 have changed since last run | `classify_all_changes` is called | returns 3 `ChangeClassification` entries; 47 specs are skipped |
| Source file change triggers re-validation | a spec lists `src/auth.rs` in frontmatter `files:`; that file has been modified | `classify_changes` is called for the spec | returns `ChangeClassification` with `ChangeKind::Source` in changes |
| First run (no cache) | `.specsync/hashes.json` does not exist | `HashCache::load` is called | returns empty cache; all files will be classified as changed |
| Requirements change triggers staleness | `requirements.md` companion has been updated | `classify_changes` is called for the parent spec | returns `ChangeClassification` with `ChangeKind::Requirements` |
| Design or testing companion change detected | `testing.md` or `design.md` has been modified | `classify_changes` is called for the parent spec | returns `ChangeClassification` with `ChangeKind::Companion` |
| Warm snapshot replay | no bound input changed after a warning-only cold check | run the same non-strict JSON check | identical diagnostics and full checked count; validated count 0 and cached count 1 |
| Snapshot invalidation | mutate a source, `.specsyncignore`, spec inventory, snapshot field, or version | run non-strict JSON check | affected specs revalidate and current diagnostics replace stale state |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Cache file missing | Returns empty cache (all files treated as changed) | Keep or add a focused assertion before changing this behavior |
| Cache file has invalid JSON | Returns empty cache silently | Keep or add a focused assertion before changing this behavior |
| Cache/snapshot version mismatch | Forces validation and rewrites current compatible state after success | `cache_format_and_snapshot_version_mismatches_force_revalidation` |
| Cached diagnostics are modified | Integrity check rejects replay | `tampered_snapshot_and_stale_inputs_cannot_produce_a_false_green` |
| Global ignore rule or spec inventory changes | Input digest invalidates every affected snapshot deterministically | `ignore_rules_and_spec_inventory_invalidate_snapshots` |
| File unreadable during hashing | `hash_file` returns `None`; file treated as changed | Keep or add a focused assertion before changing this behavior |
| Cannot create `.specsync/` directory | `save` returns `io::Error` | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/hash_cache.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
