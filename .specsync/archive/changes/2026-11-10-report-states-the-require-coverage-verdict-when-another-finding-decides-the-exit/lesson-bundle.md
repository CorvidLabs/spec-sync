# Lesson bundle: report-states-the-require-coverage-verdict-when-another-finding-decides-the-exit

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Report states the require-coverage verdict when another finding decides the exit
- **Kind**: BugFix
- **Specs**: cmd_report
- **Paths**: src/commands/report.rs, tests/integration/regression_w1.rs
- **Acceptance**: report --require-coverage N in a tree whose staleness cannot be measured still exits 1, and its text and markdown output state the coverage gate's verdict (met or not met, with the measured coverage) and name the findings that set the exit status; --require-coverage 0 reads met; a run where nothing pre-empts the gate prints exactly what it printed before

## Evidence

- Verification commit: `5ecf1d78e6efbec2fd1db2f36928a8b296d4fbd8`
- Base commit: `82c724703d5f6b89fe0a68dc672d4b1dbd8e41dc`
- Verified by: `specsync check --spec cmd_report`

## From the change's context.md

# Context

Issue #605. In a tree that is not a git repository, staleness cannot be measured and
`report` exits 1 (REQ-cmd-report-003, deliberate). Under gating enforcement that
finding exits before `--require-coverage` is consulted, so the gate never ran and
never printed a verdict. `--require-coverage 0` exited 1, and a CI reader could not
tell a coverage failure from the staleness refusal.

When another finding will decide the exit, text and markdown output now print the
coverage gate's own verdict (met or not met, with the measured figure) and name the
findings that set the exit status. Exit codes are unchanged. JSON already carries
`staleness_inconclusive` and the coverage figure, and CSV gates silently by design.

Kimi left an unapproved stub draft that planned new JSON fields
(`require_coverage`, `coverage_gate_passed`). That would grow the public contract
beyond the bug, so it was discarded. Its constraint ("the staleness refusal stays;
both gates report") is kept.

Authorization: Leif's 2026-11-10 merge rule lets bug fixes that make behavior match
existing intent (COVERAGE-2) run the full lifecycle with 0xLeif as approve and review
actor.

## From the change's testing.md

# Testing

Integration tests in `tests/integration/regression_w1.rs` run the built binary on the
minimal project with and without git history. The verdict test fails on the previous
`report.rs`.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-cmd-report-003 | integration `report_states_the_coverage_verdict_when_staleness_decides_the_exit`, `report_prints_no_extra_coverage_verdict_when_the_gate_decides_the_exit`, `report_require_coverage_above_actual_exits_1`, `report_require_coverage_at_actual_exits_0`; the staleness_unmeasurable report tests are unchanged and pass |

## Where these lessons go

- `specs/cmd_report/cmd_report.3md`
