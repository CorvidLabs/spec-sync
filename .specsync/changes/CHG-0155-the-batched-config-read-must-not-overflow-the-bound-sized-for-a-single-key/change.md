---
id: CHG-0155-the-batched-config-read-must-not-overflow-the-bound-sized-for-a-single-key
state: implementing
type: feature
base_commit: 76ef32b1ab2ce13ffdc40445dfb89b58fbf6c7cb
---

# The batched config read must not overflow the bound sized for a single key

## Intent

the batched config read must not overflow the bound sized for a single key

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Reading the effective checkout overrides succeeds when the four core keys are set in more than one configuration scope, which is the ordinary layout of a global file plus a repository-local override and already exceeds the bound that was sized for a single key's output; the values derived still equal what a separate per-key query returns for each key, asserted against that query rather than against an assumption about which scope wins; and a genuinely unbounded response is still refused, so raising the bound does not remove the deterministic-output guard.

## No-spec Rationale

Not applicable
