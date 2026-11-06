---
id: record-relayed-human-decisions-on-approvals-and-reviews-as-claims-not-authentication
state: draft
type: feature
base_commit: b179a4f254af1ddada0671eece1f8616729e2eff
---

# Record relayed human decisions on approvals and reviews as claims, not authentication

## Intent

Record relayed human decisions on approvals and reviews as claims, not authentication

## Affected Canonical Specs

- `change`
- `cmd_change`
- `cli_args`

## Acceptance Criteria

- An approval tells me it matches the content it approved and nothing more; the name on it is a claim, not proof of who gave it.
- When an agent records a decision a person made somewhere else, I can see who decided, their exact words, where they said it, and which agent wrote it down.
- That record is optional, so a person approving at their own terminal is not asked for one.
- The change's status shows it beside the approval or review it belongs to.
- The status says plainly that the record is a claim too, not proof that the person decided.
- A record missing one of those details is refused before anything is written, and the refusal says which detail is missing.
- A review an agent records for a person can carry the same record, shown the same way.

## No-spec Rationale

Not applicable
