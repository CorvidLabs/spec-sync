---
change: close-changes-that-will-not-ship
artifact: testing
---

# Testing

tests/integration/change_close.rs, through the binary:

- `closing_several_changes_moves_each_package_and_every_verb_names_the_close`: a draft and an
  approved change closed together; packages moved with `state.json` bytes kept; `close.json`
  fields; status (text and JSON), show and ship-status name the close; check refuses it; list
  omits it; `change new` refuses the reserved ID.
- `a_close_that_cannot_close_every_change_closes_none`: one unknown id closes nothing; a blank
  reason is refused.
- `a_close_records_its_replacement_and_a_relayed_decision`: an unknown `--replaced-by` is
  refused; a valid one and the relay are recorded and reported.
- `a_change_whose_materialized_spec_text_has_not_merged_is_refused_until_it_is_restored`: with
  an `origin`, a checked (materialized) change is refused naming the checkout for the spec and
  removal for the created requirements file; after both, it closes.
- Doctor (late close) unit and integration tests stay green; fmt and clippy clean.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-change-159` | `closing_several_changes_moves_each_package_and_every_verb_names_the_close`, `a_close_that_cannot_close_every_change_closes_none`, `a_close_records_its_replacement_and_a_relayed_decision`, `a_change_whose_materialized_spec_text_has_not_merged_is_refused_until_it_is_restored` in `tests/integration/change_close.rs` |
| `REQ-cmd-change-037` | `closing_several_changes_moves_each_package_and_every_verb_names_the_close`, `a_close_records_its_replacement_and_a_relayed_decision` in `tests/integration/change_close.rs` |
