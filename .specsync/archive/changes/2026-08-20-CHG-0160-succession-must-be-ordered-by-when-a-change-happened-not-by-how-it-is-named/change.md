---
id: CHG-0160-succession-must-be-ordered-by-when-a-change-happened-not-by-how-it-is-named
state: archived
type: refactor
base_commit: e8d84107eee2aafd6d61586eea87612aa0842a4a
---

# Succession must be ordered by when a change happened, not by how it is named

## Intent

succession must be ordered by when a change happened, not by how it is named

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Succession ordering is derived from the change ID string by succession_change_key, which returns (change_sequence(id), id). That silently means 'has a bigger ordinal', so under an identity scheme without one it would degrade to alphabetical order with no error and no failing test: retire-auth would have happened before add-billing because a sorts before r. It is also already wrong: validate_supersedes_edges enforces a NUMERIC strict sort while approved_scope sorts the same edges lexicographically and hashes the result into scope_digest, and those two invert at five digits, so approved_scope emits an order validate_supersedes_edges rejects. Done when: succession ordering asks when a change was created rather than what it is called; every sort over supersedes edges agrees with the one that feeds scope_digest; a predecessor created after its successor is refused whatever it is named; and no historical digest moves.

## No-spec Rationale

Not applicable
