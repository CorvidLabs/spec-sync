# Lesson bundle: change-answer-validates-question-keys-answers-and-actor-identity

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Change answer validates question keys, answers, and actor identity
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/, src/change_tests/, specs/change/, tests/
- **Acceptance**: change answer rejects an unknown question key at exit 1 naming the advertised questions; empty or whitespace-only answers are rejected; actor strings are trimmed and rejected when empty or containing newlines; affected_specs entries must name specs in the registry

## Evidence

- Verification commit: `3a513a010a67b4c69b827f93f453e7f5c90ceeb3`
- Base commit: `49d2326a8f80232b8fee94a5073c4c1a66fd2057`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

GitHub issue #426. Verified still reproducing on main 49d2326a (specsync 6.2.0):
`change answer` persists unknown question keys via the `_` arm of
`answer_question_with_snapshot` (src/change/sdd_version.rs), whitespace-only
`acceptance_criteria` answers exit 0 and store nothing (`acceptance_criteria_values`
in src/change/validate_loaded_change.rs returns an empty vec), and `--actor` with
leading/trailing whitespace or embedded newlines is stored verbatim into
approvals.json (`resolve_actor` filters empty but never trims). `affected_specs`
entries are checked for portable-name shape but not against the registry, so a
nonexistent module is accepted silently.

reopen/correct-owner/send-back already trim their actors; this change closes the
remaining paths through one central validation in `resolve_actor`.

## From the change's testing.md

# Testing

- Unit/integration: unknown question key fails and `state.json` is unchanged; a typo like `acceptance-criteria` fails.
- Empty/whitespace answers fail for each question arm.
- Approve with a padded/newline actor fails; a normal actor still works.
- `affected_specs` naming an unregistered module fails and names known modules.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-150 | integration `change_answer_rejects_unknown_question_key_and_persists_nothing`, `change_answer_rejects_empty_and_whitespace_answers`, `change_new_and_answer_reject_unregistered_affected_spec`, `change_approve_trims_and_validates_actor_identity`; full unit suite 2972/2972 and integration 581/581 green |

## Where these lessons go

- `specs/change/change.3md`
