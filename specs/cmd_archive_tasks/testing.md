---
spec: cmd_archive_tasks.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/archive_tasks.rs` | cargo test commands::archive_tasks | Command wrapper has no inline tests (output formatting only); cover `cmd_archive_tasks` end-to-end before risky changes |
| `src/archive.rs` (delegate logic) | cargo test archive | `test_archive_completed_tasks`, `test_archive_no_completed`, `test_archive_preserves_existing` |

## Coverage Gaps

- No end-to-end CLI test asserts the wrapper's stdout (per-file lines, "would archive" vs "archived", the summary line, or the "No completed tasks to archive." path). Add one before changing user-visible output.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Tasks archived successfully | a companion `tasks.md` has checked items (`- [x]`) | `cmd_archive_tasks(root, false)` is called | checked items move to `## Done`; per-file line + summary printed |
| Dry run | `tasks.md` has completed items | `cmd_archive_tasks(root, true)` is called | prints "Dry run" banner and "would archive" lines, modifies no files |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| No `tasks.md` files / no completed tasks (empty result) | Prints "No completed tasks to archive." and returns, no summary | Keep or add a focused assertion before changing this behavior |
| Multiple affected files | Summary sums `archived_count` across all results and reports the file count | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- archive-tasks --help` and confirm the help text still names the documented flags and behavior.
- Run `cargo test archive` when changing the delegate; run `cargo test commands::archive_tasks` when changing the wrapper.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an output string changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
