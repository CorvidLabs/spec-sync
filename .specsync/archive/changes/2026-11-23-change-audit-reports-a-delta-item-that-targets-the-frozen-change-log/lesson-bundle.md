# Lesson bundle: change-audit-reports-a-delta-item-that-targets-the-frozen-change-log

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Change audit reports a delta item that targets the frozen Change Log
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/historical_definition_digest_matches.rs, src/change_tests/stale_modified_delta.rs, specs/change/, CHANGELOG.md
- **Acceptance**: (1) change audit reports, for an approved, implementing or verifying change, the Change Log refusal change check gives, prefixed by the change id, and change audit --strict exits 1. (2) An archived change is not judged again. (3) A project with no such item audits as before.

## Evidence

- Verification commit: `d0876030109e7e2d082d593210470798a8727328`
- Base commit: `2cc0897fc00eee7db2b07f5134ba05639af39018`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Issue #1090, filed from fable-lead's note on #1088's review. With #1088, `change approve`, `change check`, verification and acceptance refuse a delta item that targets the frozen `## Change Log` (REQ-change-159), but the project audit (`check_project_with_command_output`, behind `change audit`) never asks that refusal, so `change audit --strict` passes an approved change that `change check` then refuses. Leif approved the definition (ss-lead/gates/1090-definition.md) "Approve as written" on 2026-11-19 at about 14:19Z, relayed by hc t4c through ss-lead hc t58; it was to be built once #1088 merged (ee76ef1b on main).

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-170 | `src/change_tests/stale_modified_delta.rs`: `audit_reports_a_change_log_delta_item_of_an_active_change`, `audit_does_not_judge_an_archived_change_log_item_again`, `audit_reports_the_change_log_item_before_a_stale_base`, and the audit's strict failure |

## How it runs

Unit tests, red on main 2cc0897f and then green; the full verify lane; and `change audit --strict` on the real repositories.

## Where these lessons go

- `specs/change/change.3md`
