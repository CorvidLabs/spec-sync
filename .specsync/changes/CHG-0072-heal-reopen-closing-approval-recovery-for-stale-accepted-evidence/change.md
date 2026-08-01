---
id: CHG-0072-heal-reopen-closing-approval-recovery-for-stale-accepted-evidence
state: verifying
type: bug_fix
base_commit: 9a00223bf254e79bd38fb41d3e2fc302edb66f71
---

# Heal reopen closing-approval recovery for stale accepted evidence

## Intent

Heal reopen closing-approval recovery for stale accepted evidence

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Accepted changes can reopen after verification tip drift or finalization closing; re-accept/finalize and archive work; definition can be re-approved while accepted when stale; legacy finalize refuses with accept+archive guidance.

## No-spec Rationale

Not applicable
