---
change: a-sent-back-no-spec-change-can-declare-that-it-changes-specs
artifact: requirements
---

# Requirements

ADDED REQ-change-160 (`deltas/change.md`): on a draft, `change answer <id> no_spec_change no`
clears `no_spec_change` and its rationale; approved changes are sent back first; `yes` and a
redundant `no` are refused.

Number allocated by claude-code-t370. Wording sent to claude-code-t370 to check against Leif's
03:52Z pick.

Evidence: `change::tests::a_sent_back_no_spec_change_can_declare_that_it_changes_specs` and
`change::tests::a_no_spec_change_answer_is_refused_where_it_cannot_apply`
(`src/change_tests/send_back_for_rescope.rs`). Red on main: the question is unknown.
