---
change: show-the-steps-between-change-new-and-approve-on-the-landing-page-and-name-the-change-answer-command-in-approve
artifact: testing
---

# Testing

All three are in `tests/integration/documented_workflows.rs` and run against the built binary.

- `landing_page_change_workflow_runs_from_its_quick_start` fails at 9c28b332 (on #881's archive tip): the landing page's approve line fails when run as written, right after `change new`.
- `approve_refusals_after_the_landing_page_change_new_name_the_command_that_finishes_them` fails at 9c28b332: `error: at least one acceptance criterion is required` names no command.
- `readme_change_workflow_archives_before_it_tells_the_reader_to_wait_for_ci` keeps passing on the shared harness.
- `.github/scripts/test-classify-ci-paths.sh` fails at 9c28b332: `site/src/content/docs/index.md` classifies `full=false`.

After the change, all of these pass, along with `fledge lanes run verify` on the pinned rust 1.89.0 and `specsync check --strict`.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-133 | `approve_refusals_after_the_landing_page_change_new_name_the_command_that_finishes_them`, `landing_page_change_workflow_runs_from_its_quick_start` |
