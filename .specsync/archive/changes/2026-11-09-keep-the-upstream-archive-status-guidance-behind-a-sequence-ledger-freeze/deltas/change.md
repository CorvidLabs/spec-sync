## ADDED

### REQUIREMENT REQ-change-138

The `change status` summary SHALL derive the handoff for a change archived upstream in the same priority order as `next_action`: a sequence-ledger freeze, then a `verification_ignored_paths` refusal, then the upstream archive. A freeze SHALL never be reported behind the upstream-archive rebase step.

Acceptance Criteria
- With an upstream archive and a frozen ledger, `next_action` and `handoff.before_clearing` both name the freeze.
- Without a freeze, both name the upstream archive.
