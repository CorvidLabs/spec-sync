---
spec: archive.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/archive.rs` | cargo test archive:: | `test_archive_completed_tasks`, `test_archive_no_completed`, `test_archive_preserves_existing` |

## Coverage Gaps

- Integration gap: add a fixture for "Archive completed tasks" before changing user-visible CLI output, generated files, or error handling in archive.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Archive completed tasks | a tasks.md file with 3 completed and 2 pending items | `archive_tasks(root, specs_dir, false)` is called | the 3 completed items move to `## Archive`, 2 pending items remain in place |
| Dry run | tasks.md files with completed items | `archive_tasks(root, specs_dir, true)` is called | returns `ArchiveResult` entries but does not modify any files |
| No completed tasks | all tasks.md files have only pending items | `archive_tasks` is called | returns an empty vec |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| tasks.md file unreadable | Prints error in red, continues processing other files | Keep or add a focused assertion before changing this behavior |
| tasks.md file unwritable | Prints error in red, continues processing other files | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/archive.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
