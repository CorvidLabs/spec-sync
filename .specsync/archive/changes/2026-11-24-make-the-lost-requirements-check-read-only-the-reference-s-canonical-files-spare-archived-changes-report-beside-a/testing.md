---
change: make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a
artifact: testing
---

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
