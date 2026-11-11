---
id: change-answer-validates-question-keys-answers-and-actor-identity
state: archived
type: bug_fix
base_commit: 49d2326a8f80232b8fee94a5073c4c1a66fd2057
---

# Change answer validates question keys, answers, and actor identity

## Intent

change answer validates question keys, answers, and actor identity

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- change answer rejects an unknown question key at exit 1 naming the advertised questions; empty or whitespace-only answers are rejected; actor strings are trimmed and rejected when empty or containing newlines; affected_specs entries must name specs in the registry

## No-spec Rationale

Not applicable
