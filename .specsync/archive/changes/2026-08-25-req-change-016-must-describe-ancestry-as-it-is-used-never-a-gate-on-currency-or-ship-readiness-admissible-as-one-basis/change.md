---
id: req-change-016-must-describe-ancestry-as-it-is-used-never-a-gate-on-currency-or-ship-readiness-admissible-as-one-basis
state: archived
type: bug_fix
base_commit: e82542d19ce8d79926b144a0e38d4d620b120715
---

# REQ-change-016 must describe ancestry as it is used: never a gate on currency or ship readiness, admissible as one basis for archival anchoring

## Intent

REQ-change-016 must describe ancestry as it is used: never a gate on currency or ship readiness, admissible as one basis for archival anchoring

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- REQ-change-016 in specs/change/requirements.md no longer claims verification.commit is never a gate unconditionally: the claim is scoped to verification currency and ship readiness, and the requirement states that archival authentication of accepted evidence is a separate question that MAY consult commit ancestry as one basis among the integrated accepted workspace and the acceptance recorded on the remote default branch.
- The requirement carries one testable obligation the code can be measured against: ancestry MUST NOT be the only basis on which anchoring can be established.
- The materialized REQ-change-016 body in specs/change/requirements.md is byte-identical to the ### REQUIREMENT REQ-change-016 body in the semantic delta, and every other requirement in that file is untouched.
- No source behaviour changes: verification_commit_is_accepted_current keeps all three call sites, two hard conjuncts in staged_accepted_snapshot_is_closing_authenticated and one of three disjuncts in accepted_evidence_is_anchored. The two conjuncts still violate the new MUST NOT clause; that violation is tracked separately as #706 and is deliberately out of scope here.

## No-spec Rationale

Not applicable
