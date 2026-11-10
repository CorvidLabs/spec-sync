---
id: every-actor-accepting-verb-normalizes-identity-the-same-way
state: implementing
type: bug_fix
base_commit: 9ef1a373f316ae8b320f4ffc0b18bc31ed0cf9f2
---

# Every actor-accepting verb normalizes identity the same way

## Intent

every actor-accepting verb normalizes identity the same way

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- send-back, reopen, correct and correct-owner reject actor names containing newlines or control characters and trim padding, exactly as approve/review do through resolve_actor

## No-spec Rationale

Not applicable
