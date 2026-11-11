---
change: change-answer-validates-question-keys-answers-and-actor-identity
artifact: context
---

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
