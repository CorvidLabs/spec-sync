---
change: a-sent-back-no-spec-change-can-declare-that-it-changes-specs
artifact: testing
---

# Testing

src/change_tests/send_back_for_rescope.rs:

- `a_sent_back_no_spec_change_can_declare_that_it_changes_specs`: a no-spec change is approved,
  sent back, answered `no_spec_change no` (flag and rationale cleared), given a Public API delta,
  approved, materialized and verified; the spec carries the delta. Red on main: the question is
  unknown.
- `a_no_spec_change_answer_is_refused_where_it_cannot_apply`: `yes` is refused naming
  `--no-spec-change --rationale`; a second `no` is refused as already cleared; an approved change
  is refused until sent back.
- Existing interview and send-back tests stay green; fmt and clippy clean.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-change-160` | `a_sent_back_no_spec_change_can_declare_that_it_changes_specs`, `a_no_spec_change_answer_is_refused_where_it_cannot_apply` in `src/change_tests/send_back_for_rescope.rs` |
