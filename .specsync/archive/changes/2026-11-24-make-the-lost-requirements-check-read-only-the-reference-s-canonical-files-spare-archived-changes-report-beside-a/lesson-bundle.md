# Lesson bundle: make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Make the lost-requirements check read only the reference's canonical files, spare archived changes, report beside a change's own errors, and name an unreadable path
- **Kind**: BugFix
- **Specs**: change, cmd_change
- **Paths**: src/change/vanished_requirements.rs, src/change/historical_definition_digest_matches.rs, src/change_tests.rs, src/change_tests/vanished_requirements.rs, tests/integration.rs, tests/integration/, specs/change/, specs/cmd_change/, site/src/content/docs/deltas.md, CHANGELOG.md
- **Acceptance**: (1) REQ-change-175: the files read at the reference come only from what was canonical there; a branch that registers a spec beside, or moves specs_dir onto, a requirements.md that was already there and rewrites it is not refused for its old ids; a requirement moved into a newly canonical file is found there. (2) REQ-change-176: change audit reports the lost requirements for each active change whether or not its approval, delta files and delta bodies validate, beside its own error. (3) REQ-change-177: when a requirement would be reported lost and a requirements directory or file in the working tree could not be read, the check says it cannot tell, naming each path and the error and the way out; nothing changes when nothing is lost. (4) REQ-cmd-change-045: change ship-status lists the lost-requirements blocker only for a change that can still be checked, finalized or shipped, not for an archived or retired one.

## Evidence

- Verification commit: `482847b4352f4241abb13c040b8bf723b1f855c5`
- Base commit: `ed258f58d734be7c4c0ac092ddbc934f492630d3`
- Verified by: `specsync check --spec change --spec cmd_change`

## From the change's context.md

# Context

Four follow-ups to #1081's lost-requirements check (REQ-change-169, REQ-cmd-change-042), filed from Codex's third and fourth review rounds on #1108 and kept out of that PR under the two-round cap: #1112 (the files read at the reference come from the working tree's scope too), #1113 (`change ship-status` lists the blocker for archived and retired changes), #1116 (`change audit` skips the check for a change whose approval or delta does not validate) and #1117 (an unreadable requirements directory is reported as a loss).

Definition: defs/1112-1113-1116-1117-definition.md (the specsync agent's draft). Confirmed by t4c under Leif's standing OK for scope approvals (2026-11-20 dialog, 'Standing OK, tell me after') at about 14:50Z: the recommended choices taken, no new refusal, all four requirements ADDED. Stacked on #1108 (`ed258f58`), like #1122. #1105 (REQ-change-174) later runs the comparison once for the whole project; the #1116 test stays valid under it.

## From the change's design.md

# Design

- **#1112 (REQ-change-175).** `lost_requirements_check` builds the reference's scope on its own (`add_reference_requirement_scope` into an empty `RequirementScope`). Only its pathspecs name the files diffed and read at the reference. Presence in the working tree keeps the union of today's scope and the reference's, so a requirement moved into a newly canonical file is still found.
- **#1113 (REQ-cmd-change-045).** `lost_requirements_refusal`, which only `change ship-status` calls, returns nothing for an archived or retired change. A draft, approved, implementing, verifying or accepted change keeps the blocker.
- **#1116 (REQ-change-176).** In `change audit`, the loss check for an approved, implementing or verifying change runs on its own, no longer behind the approval, delta-file, delta-body and delta-base checks. An approval or delta that cannot be read already accounts for no removal (`add_approved_removals`).
- **#1117 (REQ-change-177).** `working_tree_requirement_blocks` also returns each scope directory it cannot list and each requirements file it cannot read, for any reason other than not existing. When the check would report a loss and that list is not empty, it refuses with the existing "cannot tell whether this branch lost canonical requirements" form, naming each path with its error and the way out: make it readable, then run `specsync change check <id>` again. With nothing lost it says nothing.

## From the change's testing.md

# Testing

## Requirement evidence

Each acceptance criterion and its test, all in `tests/integration/vanished_requirements.rs`:

| Requirement | Criterion | Evidence |
|---|---|---|
| REQ-change-175 | 1. a spec registered beside an existing `requirements.md` | `a_file_the_branch_makes_canonical_is_not_read_at_the_reference` |
| REQ-change-175 | 2. `specs_dir` moved onto an existing `requirements.md` | `a_specs_directory_the_branch_moves_onto_is_not_read_at_the_reference` |
| REQ-change-175 | 3. moved into a newly canonical file is found; dropped, it is reported | `a_requirement_moved_into_a_newly_canonical_file_is_found_and_reported_once_dropped` |
| REQ-change-175 | nothing canonical at the reference reads nothing there | `a_reference_whose_specs_dir_leaves_the_project_is_not_read_whole`, `a_reference_whose_configuration_does_not_parse_is_not_read_whole` |
| REQ-change-176 | 1. a delta that does not parse | `the_audit_reports_a_lost_requirement_beside_a_malformed_delta` |
| REQ-change-176 | 2. an invalid approval; a delta body edited after approval | `the_audit_reports_a_lost_requirement_beside_an_unreadable_approval`, `the_audit_reports_a_lost_requirement_beside_an_edited_delta_body` |
| REQ-change-176 | 3. nothing lost: only the change's own error | #1081's audit tests, unchanged |
| REQ-change-177 | 1. a directory the check cannot list | `an_unreadable_requirements_directory_is_named_instead_of_a_loss`, `a_scope_directory_below_an_unreadable_directory_is_named_instead_of_a_loss` |
| REQ-change-177 | 2. readable again, the check passes | `an_unreadable_requirements_directory_is_named_instead_of_a_loss` (its last step) |
| REQ-change-177 | 3. a requirements file that cannot be read | `an_unreadable_requirements_file_is_named_instead_of_a_loss` |
| REQ-change-177 | 4. nothing lost, an unreadable directory changes nothing | `an_unreadable_directory_with_nothing_lost_changes_nothing` |
| REQ-cmd-change-045 | 1. an archived change lists no blocker | `ship_status_of_an_archived_change_lists_no_lost_requirement` |
| REQ-cmd-change-045 | 2. an active change on the same branch still lists it | `ship_status_of_an_archived_change_lists_no_lost_requirement` (the draft beside it) |

All seven integration tests fail on #1108's head `ed258f58` before the fix. Then the full lane, `change audit --strict`, and the real repositories (rule 3): starling's PR #43 shape, the private peck repository, and the spec-sync and Chough mains, each giving #1108's verdict except where an issue says it should differ.

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
