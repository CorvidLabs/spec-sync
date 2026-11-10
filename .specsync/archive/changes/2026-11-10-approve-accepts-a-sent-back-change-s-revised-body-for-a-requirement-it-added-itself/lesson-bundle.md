# Lesson bundle: approve-accepts-a-sent-back-change-s-revised-body-for-a-requirement-it-added-itself

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Approve accepts a sent-back change's revised body for a requirement it added itself
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/collect_requirement_ids.rs, src/change_tests/send_back_for_rescope.rs
- **Acceptance**: after a send-back, change approve accepts a revised body for an ADDED requirement whose key the send-back recorded as this change's own applied item while the canonical text is still that item's text, and materialization then replaces the block; an ADDED requirement whose canonical text someone else changed is still refused with 'cannot add existing block'

## Evidence

- Verification commit: `cf7f52fba17f60e667e8917ff76171d8a0894702`
- Base commit: `82c724703d5f6b89fe0a68dc672d4b1dbd8e41dc`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Issue #1001, found while re-scoping #994 after a send-back. Materialization
(`prepare_module_application`) treats a sent-back change's own earlier ADDED block as
MODIFIED when `sent_back_added_block_is_own` finds the send-back listed it as applied
and the canonical tree still carries that text (#914/#960). `change approve` runs
`validate_delta_files` first, which refused any ADDED requirement whose living text
differs ("cannot add existing block"), with no such exception. A sent-back change
therefore could not be re-approved with a revised body for a requirement it added
itself. The #994 workaround deleted the materialized blocks by hand.

`validate_delta_files` now asks the same helper before refusing, reading the same
ledger and canonical files materialization reads. A block someone else changed is
still refused.

Authorization: Leif's 2026-11-10 merge rule lets bug fixes that make behavior match
existing requirements (here REQ-change-123's send-back criterion) run the full lifecycle
with 0xLeif as approve and review actor.

## From the change's testing.md

# Testing

Unit tests in `src/change_tests/send_back_for_rescope.rs` drive approve, materialize,
send-back, a revised delta and re-approval on a temp project. The positive test fails on
the previous code with the #1001 error.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-123 | unit `a_sent_back_change_can_be_re_approved_with_a_revised_requirement_it_added`, `approve_still_refuses_an_added_requirement_whose_canonical_text_someone_else_changed`, and the existing `a_sent_back_change_can_change_the_body_of_an_item_it_added` |

## Where these lessons go

- `specs/change/change.3md`
