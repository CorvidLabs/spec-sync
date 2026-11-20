---
change: make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-175 | `tests/integration/vanished_requirements.rs`: `a_file_the_branch_makes_canonical_is_not_read_at_the_reference`, `a_specs_directory_the_branch_moves_onto_is_not_read_at_the_reference`; #1081's moved-requirement and dropped-requirement tests stay green |
| REQ-change-176 | `the_audit_reports_a_lost_requirement_beside_a_malformed_delta`, `the_audit_reports_a_lost_requirement_beside_an_unreadable_approval` |
| REQ-change-177 | `an_unreadable_requirements_directory_is_named_instead_of_a_loss`, `an_unreadable_requirements_file_is_named_instead_of_a_loss`; `an_unreadable_directory_with_nothing_lost_changes_nothing` |
| REQ-cmd-change-045 | `ship_status_of_an_archived_change_lists_no_lost_requirement`, with #1081's `ship_status_lists_a_lost_requirement_and_ship_and_finalize_refuse_it` for the active change |

All seven integration tests fail on #1108's head `ed258f58` before the fix. Then the full lane, `change audit --strict`, and the real repositories (rule 3): starling's PR #43 shape, the private peck repository, and the spec-sync and Chough mains, each giving #1108's verdict except where an issue says it should differ.
