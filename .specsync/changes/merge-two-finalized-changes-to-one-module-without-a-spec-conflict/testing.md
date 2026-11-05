---
change: merge-two-finalized-changes-to-one-module-without-a-spec-conflict
artifact: testing
---

# Testing

`parallel_finalized_module_changes_merge_cleanly_in_either_order` builds two branches in the shape materialize writes and runs `git merge-tree` both ways. Before the driver existed, `merge --install` was rejected and the test failed. After the fix it is clean, `version:` is 7 when both sides bumped 5 to 6, and both orders produce the same spec and requirements text.

`duplicate_requirement_id_is_refused_by_check_and_change_audit` writes `REQ-auth-002` twice. Before the fix, `check` passed. After the fix, both `check` and `change audit` exit non-zero and the output contains `REQ-auth-002`.

Unit tests on the driver cover commutative bumps, unequal version numbers, quoted versions, duplicate requirement bodies kept twice, divergent purpose left in conflict, and a one-sided edit returned byte-for-byte. The existing `merge::tests` suite still passes, including `frontmatter_version_takes_max_not_incoming`.

`merge_install_refuses_a_symlinked_gitattributes` points `.gitattributes` at another file and runs `merge --install`. The command fails and names the refusal, the link target keeps its bytes, and no `merge.specsync.driver` is recorded.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-merge-004 | `parallel_bumps_merge_commutatively_and_count_both`, `different_version_numbers_still_merge_to_one_line`, `quoted_version_keeps_its_quotes_when_both_bumps_count`, `one_sided_edit_is_returned_unchanged`, `parallel_finalized_module_changes_merge_cleanly_in_either_order`, `frontmatter_version_takes_max_not_incoming` |
| REQ-merge-005 | `divergent_prose_stays_a_conflict`, `requirement_additions_merge_in_either_order_and_duplicate_ids_are_kept` |
| REQ-cmd-merge-003 | `parallel_finalized_module_changes_merge_cleanly_in_either_order`, `merge_install_refuses_a_symlinked_gitattributes` |
| REQ-cli-012 | `parallel_finalized_module_changes_merge_cleanly_in_either_order` (runs `merge --install`; `git merge-tree` invokes `merge-file` through the installed driver) |
| REQ-cli-args-023 | `parallel_finalized_module_changes_merge_cleanly_in_either_order`, `merge_install_refuses_a_symlinked_gitattributes` (both parse `merge --install --root <dir>`) |
| REQ-validator-045 | `names_a_repeated_heading_and_ignores_a_fenced_example`, `a_single_heading_is_not_a_duplicate`, `duplicate_requirement_id_is_refused_by_check_and_change_audit` |
| REQ-change-125 | `duplicate_requirement_id_is_refused_by_check_and_change_audit` |
