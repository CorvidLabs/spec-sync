---
spec: archive.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/archive.rs` | `fledge run test -- archive::tests` | Parsing, all/mixed plan failure, middle-stage failure, middle-publish rollback, rollback failure/partial state, dry-run/apply parity, and permission preservation |
| `tests/integration/commands.rs` | `fledge run test -- archive_tasks_` | CLI preview plus parse-clean exit-1 failure report and zero-write assertion |

## Coverage Gaps

(none for the transactional archive-tasks remediation)

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Archive completed tasks | a tasks.md file with 3 completed and 2 pending items | `archive_tasks(root, specs_dir, false)` is called | the 3 completed items move to `## Archive`, 2 pending items remain in place |
| Dry run | tasks.md files with completed items | `archive_tasks(root, specs_dir, true)` is called | returns `ArchiveResult` entries but does not modify any files |
| No completed tasks | all tasks.md files have only pending items | `archive_tasks` is called | returns an empty vec |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| One candidate is unreadable/non-UTF-8 | Returns a read failure and modifies zero destinations | `planning_failure_prevents_all_destination_writes`, `archive_tasks_apply_failure_exits_one_and_reports_zero_writes` |
| Middle candidate cannot be staged | Drops all staged temporaries and publishes zero destinations | `middle_staging_failure_prevents_all_destination_writes` |
| Middle publication fails | Reports publish failure and restores prior destinations | `middle_publish_failure_rolls_back_prior_replacements` |
| Rollback also fails | Leaves the unrestored operation in `succeeded` and exposes `partial: true` | `rollback_failure_exposes_the_remaining_partial_apply` |
| Destination has non-default permissions | Atomic replacement preserves the original permission bits | `apply_preserves_original_unix_permissions` |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/archive.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an operation name or error schema changes, update the matching Regression Matrix row and structured-output assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
