---
change: refuse-a-stale-modified-delta-instead-of-overwriting-newer-canonical-text
artifact: testing
---

# Testing

`a_stale_modified_delta_never_overwrites_text_main_added_after_materialization` and `a_stale_modified_delta_never_overwrites_text_main_added_before_the_first_check` fail on main f82eb09, which overwrites main's text and returns Ok. They pass with the fix. Run with `cargo test --bin specsync stale_modified`.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-122 | `a_stale_modified_delta_never_overwrites_text_main_added_after_materialization`, `a_stale_modified_delta_never_overwrites_text_main_added_before_the_first_check`, `audit_reports_a_stale_modified_delta`, `a_rebased_and_re_approved_delta_materializes_over_the_moved_requirement`, `main_moving_an_unrelated_block_does_not_refuse_the_check`, `an_approval_without_recorded_bases_keeps_materializing` |
