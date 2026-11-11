# Lesson bundle: a-lifecycle-rewrite-keeps-the-fields-a-newer-specsync-wrote

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: A lifecycle rewrite keeps the fields a newer specsync wrote
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change.rs, src/change/unknown_members.rs, src/change/doctor.rs, src/change/historical_definition_digest_matches.rs, src/change/historical_reconciliation.rs, src/change/relayed_decision.rs, src/change/retirement.rs, src/change/sdd_version.rs, src/change/send_back.rs, src/change/validate_loaded_change.rs, src/change_tests.rs, src/change_tests/unknown_members.rs, tests/integration/change.rs
- **Acceptance**: A lifecycle record or ledger SpecSync rewrites in place keeps every member it does not know, with its exact text, at the top level and inside entries the rewrite left unchanged, through send-back, answer, approve, check, review, ship and relay repair, and the audit stays clean

## Evidence

- Verification commit: `0af6bada3b51e4b908ff9c901479c6cd34854107`
- Base commit: `46f55652649a72ddf5e1715938b504b96686566b`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Issue #1043, found while working PR #1037: one send-back by the installed 6.2.0 removed four relayed
approval records from an active change. Coordinator t370 (2026-11-11) chose preserve over refuse,
because refusing contradicts hi CHANGE-13.h, and asked for one change that also fixes #434 part 1.
The delta adds requirement text, so the change stops at READY for Leif's definition approval.

## From the change's design.md

# Design

`json_content_keeping_unknown_members(previous, value)` (src/change/unknown_members.rs) replaces
`json_content` wherever a record or ledger is rewritten in place. It parses three documents into
spans of their JSON text: the file on disk, the typed round trip of that file (what this binary
knows), and the new typed value.

- An object member present on disk and absent from the round trip is unknown. It is appended, with
  its exact text, after the last member of the same object in the new text, unless the new object
  already has that key. A known member the rewrite removed stays removed.
- The root is the same record or ledger before and after.
- An array entry is matched to the new entry with the same known text, preferring the same position,
  so an insertion, a removal or a sort keeps every entry's members on that entry. An entry left
  unmatched in an array whose length did not change is treated as the same record edited in place
  (a relay repair edits one approval), and keeps its members. A different record replacing it at the
  same position would inherit them; no lifecycle rewrite does that, and the acceptance line says so. An entry that holds unknown members and still
  matches nothing was removed or replaced: the rewrite is refused rather than dropping them.
- Key tokens are copied byte for byte, so an escaped key keeps its exact text.
- The output keeps the layout `serde_json::to_string_pretty` writes; known members keep their order.
- It must parse back to exactly the new value; otherwise the write is refused naming the file and
  the members. A missing or unreadable previous file writes `json_content`'s bytes unchanged.

Fresh records (a new `verification.json`, a new `review.json`, receipts, finalization records)
keep `json_content`. Digests are computed from typed values, so none changes.

## From the change's testing.md

# Testing

Unit tests in `src/change_tests/unknown_members.rs` and one end-to-end CLI test.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-079 | unit `an_older_specsync_appending_to_a_ledger_keeps_what_a_newer_one_wrote`, `an_entry_the_rewrite_changed_or_replaced_never_gains_another_records_members`, `kept_members_keep_their_exact_text_and_known_members_their_order`, `a_rewrite_with_nothing_unknown_writes_exactly_what_it_wrote_before`, `a_known_field_the_rewrite_sets_or_removes_is_never_overridden_by_the_old_text`, `an_empty_object_in_the_new_text_still_takes_its_unknown_members`; integration `lifecycle_rewrites_keep_members_a_newer_specsync_wrote` (fails on main at the first send-back) |

## Where these lessons go

- `specs/change/change.3md`
