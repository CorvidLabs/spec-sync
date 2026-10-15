---
spec: watch.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/watch.rs` (event filtering) | cargo test watch:: | `test_is_relevant_event_create`, `test_is_relevant_event_modify`, `test_is_relevant_event_remove`, `test_is_relevant_event_rejects_access`, `test_is_relevant_event_rejects_other`, `test_is_relevant_event_create_any` |
| `src/watch.rs` (check args) | cargo test watch:: | `test_build_check_args_basic`, `test_build_check_args_strict`, `test_build_check_args_force`, `test_build_check_args_require_coverage`, `test_build_check_args_all_flags` |
| `src/watch.rs` (watch dirs / event path) | cargo test watch:: | `test_run_watch_collects_watch_dirs`, `test_run_watch_empty_dirs_detected`, `test_event_path_extraction`, `test_event_path_extraction_no_paths` |

| Exact-source selection/modes | `cargo test watch::` | Exact `lib.rs`/`main.rs` are `NonRecursive`; source/spec directories remain `Recursive`; alias deduplication never expands the root |
| Confined path selection | `cargo test watch::` | Escape and absolute-path rejection; symlink files/directories/parent components; nonregular source rejection; changed identity on replacement |
| Exact-parent lifecycle event filtering | `cargo test watch::` | Sibling-only events are ignored; matching rename target is selected even if the temporary sibling appears first; recursive directory events remain eligible |
| Coarse immediate-parent events | `cargo test watch::` | Bounded content fingerprints detect same-size/restored-mtime source edits, atomic replacement, missing/recreated files; sibling-only events do nothing and precise events refresh state |
| Fingerprint safety | `cargo test watch::` | Oversize input reports the existing 8 MiB cap; directory/symlink replacements are invalid and never followed |
| Missing versus invalid diagnostics | `cargo test watch::` | Distinct human wording and structured JSON codes/path/role; a regular file cannot stand in for `specs_dir` |

## Coverage Gaps

- A backend event claim requires a bounded real child process, a passing initial check, a successful watched-spec edit control and a successful exact-source edit rerun. Unit selection assertions alone do not prove filesystem event delivery.
- Initial Mac probes were inconclusive; a comment-only coalesced spec-directory rerun is not a discriminating source-edit proof. The stronger two-transition fixture makes `files: []` produce a real frontmatter failure, restores the exact valid spec and requires a new passing check before source phases.
- On this Mac, both the frozen pre-followup `f72f0b6` binary and the corrected candidate detected the invalid-spec failure but timed out observing exact restoration in the same normal workspace fixture. That reproduces an inherited/local backend limitation; it does not establish a new source-specific regression. The complete Mac source/save event sequence has **not passed**.
- The bounded Linux hosted process regression remains mandatory: causal spec failure and restored pass, ordinary exact-file edit, atomic replacement and subsequent edit, unlink/recreation and subsequent edit, then sibling-negative control. A local backend timeout must not be counted as passing or silently skipped.
- Exact-file atomic replacement/removal/recreation must be tested on the native backend, along with a later edit proving continued monitoring and an unrelated sibling edit proving exact-path filtering. Do not infer complete save-strategy/platform coverage from an in-place edit.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Initial run | a project with specs and source directories | `run_watch` is called | runs `specsync check` immediately, then watches for changes |
| File modification triggers re-check | watch mode is running | a `.spec.md` file is modified | re-runs check after 500ms debounce, showing the changed file path |
| Exact regular source edit | initialized Cargo root `lib.rs`/`main.rs`, initial check passes, native event control works | edit a selected source to introduce an undocumented export | `Changed:` names that exact file, and the strict child reports the new warning/failure |
| Path safety | configured links, escapes and a nonregular source beside valid paths | start watch in text and JSON modes | invalid paths disclosed accurately; valid directories/files remain registered; no parent/root broadening |
| Rapid saves | watch mode is running | multiple files are saved within 500ms | only one check run is triggered (debounced) |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| No valid directories or exact files to watch | Prints error, exits with code 1 | Keep or add a focused assertion before changing this behavior |
| Watcher creation fails | Panics with "Failed to create file watcher" | Keep or add a focused assertion before changing this behavior |
| Individual path watch fails | Prints an accurate warning, excludes it from the banner and continues only if another path registers | Keep or add a focused assertion before changing this behavior |
| Check command fails | Prints "Some checks failed", continues watching | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/watch.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
