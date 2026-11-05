---
change: send-an-approved-change-back-to-draft-for-re-scoping
artifact: testing
---

# Testing

The first five tests in `src/change_tests/send_back_for_rescope.rs` were written before the implementation and all failed against a stub `send_back_change`. The finalize-and-audit test and the CLI grammar test came afterwards. All seven pass with the implementation.

Review round on e52f77d (SpecSync bot, REQUEST_CHANGES). Three tests were added and the finalize test was rewritten to use a change that owns a spec, before the fix; the failing-first run is in `/workspace/pr-burndown/ss846-review-failing-first.log`. Against e52f77d, `a_re_scope_that_drops_applied_text_is_refused_until_the_text_is_removed`, `the_definition_approval_event_resolver_honours_a_send_back`, `deleting_a_committed_send_back_does_not_revive_the_cancelled_approval` and the rewritten `a_sent_back_change_finalizes_and_its_archive_passes_audit` all failed. All ten pass with the fix. Run them with `cargo test --bin specsync -- send_back sent_back sending_back only_an_approved_unfinished re_scope`.

The command output was inspected on a scratch project with the built binary: text output, `--json` output carrying `change` and `audit`, and the non-zero refusal for a draft.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-123 | `a_verifying_change_sent_back_can_be_re_scoped_and_delivered`, `sending_back_needs_a_named_person_and_a_reason`, `a_send_back_is_kept_in_the_change_history_and_loses_nothing`, `sending_back_cancels_the_approval_until_a_fresh_one`, `only_an_approved_unfinished_change_can_be_sent_back`, `a_sent_back_change_finalizes_and_its_archive_passes_audit`, `a_re_scope_that_drops_applied_text_is_refused_until_the_text_is_removed`, `the_definition_approval_event_resolver_honours_a_send_back`, `deleting_a_committed_send_back_does_not_revive_the_cancelled_approval`, `a_sent_back_change_with_a_removed_item_can_be_checked_again`, `a_sent_back_removed_only_change_materializes_again`, `a_sent_back_change_can_change_the_body_of_an_item_it_added`, `a_dropped_modified_item_is_refused_with_advice_to_restore_its_earlier_text`, `a_dropped_removed_item_is_refused_until_the_section_is_restored`, `a_module_dropped_from_the_scope_is_refused_while_it_keeps_the_change_log_row`, `a_send_back_deleted_in_a_later_commit_is_refused_and_audited` |
| REQ-cmd-change-026 | Scratch-project CLI inspection of text, `--json` and refusal output |
| REQ-cli-args-021 | `change_send_back_requires_and_collects_audit_inputs` |
