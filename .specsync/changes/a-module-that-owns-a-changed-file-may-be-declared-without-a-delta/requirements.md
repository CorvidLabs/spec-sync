---
change: a-module-that-owns-a-changed-file-may-be-declared-without-a-delta
artifact: requirements
---

# Requirements

ADDED REQ-change-161 (`deltas/change.md`): a declared module with no delta is allowed when it owns
one of the change's declared files; its spec is not edited; at least one module keeps a delta;
otherwise the module is refused at approval.

Number allocated by claude-code-t370. Wording sent to claude-code-t370 to check against Leif's
03:52Z pick.

Evidence: `a_module_that_owns_a_changed_file_may_be_declared_without_a_delta` and
`a_module_with_no_delta_that_owns_none_of_the_files_is_refused`
(`src/change_tests/send_back_for_rescope.rs`).
