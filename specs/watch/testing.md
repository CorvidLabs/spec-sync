---
spec: watch.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/watch.rs` | cargo test watch:: | `test_is_relevant_event_create`, `test_is_relevant_event_modify`, `test_is_relevant_event_remove`, `test_is_relevant_event_rejects_access`, `test_is_relevant_event_rejects_other`, `test_is_relevant_event_create_any` |

## Coverage Gaps

- Integration gap: add a fixture for "Initial run" before changing user-visible CLI output, generated files, or error handling in watch.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Initial run | a project with specs and source directories | `run_watch` is called | runs `specsync check` immediately, then watches for changes |
| File modification triggers re-check | watch mode is running | a `.spec.md` file is modified | re-runs check after 500ms debounce, showing the changed file path |
| Rapid saves | watch mode is running | multiple files are saved within 500ms | only one check run is triggered (debounced) |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| No directories to watch | Prints error, exits with code 1 | Keep or add a focused assertion before changing this behavior |
| Watcher creation fails | Panics with "Failed to create file watcher" | Keep or add a focused assertion before changing this behavior |
| Individual dir watch fails | Prints warning, continues watching other dirs | Keep or add a focused assertion before changing this behavior |
| Check command fails | Prints "Some checks failed", continues watching | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/watch.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
