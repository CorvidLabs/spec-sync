---
spec: hash_cache.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/hash_cache.rs` | cargo test hash_cache:: | `cache_round_trip`, `snapshots_round_trip_and_old_caches_revalidate`, `snapshot_integrity_and_input_changes_are_rejected`, `is_changed_detects_new_file`, `is_changed_detects_modification`, `extract_files_from_frontmatter`, `prune_removes_missing`, `classify_detects_spec_change`, `classify_detects_requirements_change`, `classify_detects_companion_change`, `classify_detects_testing_companion_change`, `classify_detects_source_change`, `companion_files_found_with_plain_names`, `update_cache_tracks_plain_companion_files` |
| `src/hash_cache.rs` (REQ-hash-cache-007) | cargo test hash_cache:: | `a_link_at_the_work_tree_cache_path_is_never_followed`, `save_creates_nothing_through_a_dangling_link_or_a_linked_specsync`, `load_reads_no_cache_through_a_link`, `a_directory_at_the_cache_path_is_refused`, `save_replaces_the_cache_by_rename`. Every link points inside the test's own temporary directory |
| `tests/integration/check.rs` (REQ-hash-cache-007) | cargo test --test integration hash_cache | `check_never_writes_the_hash_cache_through_a_link`, `rehash_never_writes_the_hash_cache_through_a_link`: the real binary leaves the link's target byte for byte unchanged |
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
| `.specsync/hashes.json` or `.specsync` is a symbolic link | Nothing is read or written through it; `save` errors, `load` is empty, `refusal` names the path | Keep the six REQ-hash-cache-007 unit tests and both integration tests; they fail on the code before the fix |
| A hard link to the cache file | `save` renames a new file over the name, so the other name keeps the old bytes | Keep `save_replaces_the_cache_by_rename` |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/hash_cache.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.

## Bounded reads (REQ-hash-cache-008)

FIFOs run under a timeout and over-bound files are sparse; nothing reads `/dev/zero`. Unix-only.

| Case | Required behavior | Test |
|------|-------------------|------|
| A project file this module reads is a FIFO, a link out of the project, or over its bound | Refused without waiting or reading past the bound | `tests/integration/project_reads.rs`: `check_does_not_wait_on_a_companion_that_is_not_a_regular_file`, `check_refuses_a_hash_cache_over_its_bound`; `tests/integration/project_reads.rs`: `no_project_file_is_read_outside_the_bounded_read`, the source scan that fails on any direct file read outside the bounded project read |

## A tracked cache and prune (REQ-hash-cache-009, REQ-hash-cache-010)

Every link points inside the test's own temporary directory.

| Case | Required behavior | Test |
|------|-------------------|------|
| A cache carrying a forged pass, committed to Git | Never replayed; `save` writes the local state folder and the committed bytes stay | `src/hash_cache.rs`: `a_forged_pass_in_a_cache_git_tracks_is_never_replayed`; `tests/integration/repository_inputs.rs`: `rehash_leaves_a_hash_cache_git_tracks` |
| A forged pass in an unpacked tree, or committed as `.specsync/Hashes.json` on a case-insensitive disk | Never replayed | `src/hash_cache.rs`: `a_forged_pass_an_unpacked_tree_ships_is_never_replayed`, `a_forged_pass_committed_in_another_letter_case_is_never_replayed`; `tests/integration/repository_inputs.rs`: `check_never_uses_a_hash_cache_an_unpacked_tree_ships` |
| `prune` with an absolute key and a `..` key naming an outside file | Both dropped; a project file's key kept | `src/hash_cache.rs`: `prune_drops_keys_that_leave_the_project` |
