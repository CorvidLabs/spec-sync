---
id: failed-verification-evidence-carries-its-findings
state: implementing
type: bug_fix
base_commit: 49d2326a8f80232b8fee94a5073c4c1a66fd2057
---

# Failed verification evidence carries its findings

## Intent

failed verification evidence carries its findings

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- a failed CommandEvidence in verification.json carries a bounded output field with the findings text; passing records serialize byte-identically to before; a flake vs regression is classifiable from the artifact alone

## No-spec Rationale

Not applicable
