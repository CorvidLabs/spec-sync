---
id: status-and-show-check-every-review-attempt-s-relay-before-showing-the-claims
state: implementing
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# Status and show check every review attempt's relay before showing the claims

## Intent

Status and show check every review attempt's relay before showing the claims

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- change status and change show refuse, with the repair-relays recovery, a review-attempts.json attempt whose relay names someone other than its reviewer or is malformed, while the latest review.json is valid; a review with no attempt history still shows its claims (red then green)

## No-spec Rationale

Makes the status and show claim projection match REQ-change-relayed-decision-001: reading refuses a damaged relayed_by with its recovery. recorded_claims read only the latest review.json, so a damaged earlier attempt in review-attempts.json was shown as fine.
