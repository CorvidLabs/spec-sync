---
change: send-an-approved-change-back-to-draft-for-re-scoping
artifact: testing
---

# Testing

The first five tests in `src/change_tests/send_back_for_rescope.rs` were written before the implementation and all failed against a stub `send_back_change`. The finalize-and-audit test and the CLI grammar test came afterwards. All seven pass with the implementation. Run them with `cargo test --bin specsync -- send_back sent_back sending_back only_an_approved_unfinished`.

The command output was inspected on a scratch project with the built binary: text output, `--json` output carrying `change` and `audit`, and the non-zero refusal for a draft.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-123 | `a_verifying_change_sent_back_can_be_re_scoped_and_delivered`, `sending_back_needs_a_named_person_and_a_reason`, `a_send_back_is_kept_in_the_change_history_and_loses_nothing`, `sending_back_cancels_the_approval_until_a_fresh_one`, `only_an_approved_unfinished_change_can_be_sent_back`, `a_sent_back_change_finalizes_and_its_archive_passes_audit` |
| REQ-cmd-change-026 | Scratch-project CLI inspection of text, `--json` and refusal output |
| REQ-cli-args-021 | `change_send_back_requires_and_collects_audit_inputs` |
