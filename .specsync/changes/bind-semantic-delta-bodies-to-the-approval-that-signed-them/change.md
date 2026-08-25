---
id: bind-semantic-delta-bodies-to-the-approval-that-signed-them
state: verifying
type: bug_fix
base_commit: 875752ee991d458db172dec6ceb712462fe2a614
---

# Bind semantic delta bodies to the approval that signed them

## Intent

Bind semantic delta bodies to the approval that signed them

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- A semantic delta body swapped after approval is refused before it rewrites the canonical spec, and the refusal names the module
- An untouched approved delta still materializes into the canonical spec exactly as before
- An approval that records no delta digest — every archived change in this repository — proceeds, because absent evidence is unknown and not a violation

## No-spec Rationale

Not applicable
