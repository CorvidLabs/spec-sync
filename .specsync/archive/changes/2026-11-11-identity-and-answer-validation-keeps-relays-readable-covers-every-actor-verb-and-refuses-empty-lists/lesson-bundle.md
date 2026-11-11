# Lesson bundle: identity-and-answer-validation-keeps-relays-readable-covers-every-actor-verb-and-refuses-empty-lists

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Identity and answer validation keeps relays readable, covers every actor verb and refuses empty lists
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/sdd_version.rs, src/change/validate_loaded_change.rs, src/change/retirement.rs, src/change/doctor.rs, src/change/historical_reconciliation.rs, tests/integration/change.rs, tests/integration/relayed_decision.rs
- **Acceptance**: a relayed approval with a padded actor stays readable; retire, doctor late close, reopen and reconcile-history normalize actors before any write and reopen does so before moving an archive; Unicode line separators and bidi overrides are refused in actors; empty-list answers are refused and none clears affected_specs and dependencies; affected_specs refusals name the canonical path or the unreadable registry; REQ-change-150, invariant 52 and the lesson match the code

## Evidence

- Verification commit: `6c579737224dccc753f35c53a42104c6d11500fb`
- Base commit: `b7c48a4b945ffebda99940ff6b379896083bc73f`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

PR #1026 (Fixes #426), round 4. Leif's quality bar asks for an independent review before
merge. A fresh reviewer at `b7c48a4b` requested changes at about 40% confidence:

- Must-fix: `approve --actor "  Ada  "` with a relayed decision recorded `actor: "Ada"` beside
  `relayed_by.decided_by: "  Ada  "`, a ledger every later read refused. Fixed in
  `approve_definition_with_relay` by trimming the relay's decider the same way.
- Retire and the doctor late close skipped the shared actor rule. Reopen moved an archived
  package before checking the actor. Reconcile-history's control-character check re-judged
  committed records.
- `[]` and `,` answers silently cleared required lists. `dependencies ''` was the only way to
  clear dependencies and is now refused, so `none` clears `affected_specs` and `dependencies`.
- The `affected_specs` refusal gave one misleading message for a missing canonical spec, a
  spec outside its canonical path, and an unreadable registry.
- U+2028 and bidi overrides passed the actor check.
- REQ-change-150, invariant 52 and the z=46 lesson claimed more than the code did.

REQ-change-150 is this PR's own unmerged requirement. Its delta brings the text in line with
the code. The coordinator directed that #1026 continue as a bug fix under Leif's rule.

## From the change's design.md

# Design

One rule, one place. `normalize_actor` trims and refuses control characters, U+2028, U+2029,
U+202A to U+202E and U+2066 to U+2069. Each verb calls it at the top, before any lock-protected
move or write: approve (through `resolve_actor`), reopen, send-back, correct, correct-owner,
reconcile-history, retire and the doctor late close. Review keeps
`validate_scoped_reviewer_claim`, which is stricter. `approve_definition_with_relay` trims the
relay's decider so the equality the ledger reader enforces holds. Persisted reconciliation
records are validated as before, with no new character rule, so history is not re-judged.

List answers: `acceptance_criteria` and `affected_paths` refuse a parse with no value.
`affected_specs` and `dependencies` go through `clearable_list_answer`, where `none` clears
and an empty parse is refused with that hint.

## From the change's testing.md

# Testing

Integration tests drive the built binary on temp projects. The relay test fails with the
decider trim reverted.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-150 | integration `a_relayed_approval_with_a_padded_actor_records_one_decider_and_stays_readable`, `change_answer_refuses_empty_lists_and_none_clears_optional_lists`, `change_approve_trims_and_validates_actor_identity`, `change_new_and_answer_reject_unregistered_affected_spec`, `change_answer_rejects_unknown_question_key_and_persists_nothing`, `change_answer_rejects_empty_and_whitespace_answers` |

## Where these lessons go

- `specs/change/change.3md`
