---
change: merge-two-finalized-changes-to-one-module-without-a-spec-conflict
artifact: testing
---

# Testing

`parallel_finalized_module_changes_merge_cleanly_in_either_order` builds two branches in the shape materialize writes and runs `git merge-tree` both ways. Before the driver existed, `merge --install` was rejected and the test failed. After the fix it is clean, `version:` is 7 when both sides bumped 5 to 6, and both orders produce the same spec and requirements text.

`duplicate_requirement_id_is_refused_by_check_and_change_audit` writes `REQ-auth-002` twice. Before the fix, `check` passed. After the fix, both `check` and `change audit` exit non-zero and the output contains `REQ-auth-002`.

Unit tests on the driver cover commutative bumps, unequal version numbers, quoted versions, duplicate requirement bodies kept twice, divergent purpose left in conflict, and a one-sided edit returned byte-for-byte. The existing `merge::tests` suite still passes, including `frontmatter_version_takes_max_not_incoming`.
