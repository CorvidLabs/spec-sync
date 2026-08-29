---
id: ship-status-readiness-must-ask-whether-the-scoped-review-is-current-and-say-so-when-it-cannot-tell
state: implementing
type: bug_fix
base_commit: 7df407728de3ac6458ef8807e79bbadb51da3324
---

# Ship-status readiness must ask whether the scoped review is current, and say so when it cannot tell

## Intent

Ship-status readiness must ask whether the scoped review is current, and say so when it cannot tell

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- ship-status readiness consults scoped-review currency, reports it as current/stale/unavailable, blocks on a decided stale review naming what moved, never reports an unavailable guarantee as satisfied, and agrees with finalize on the same tree; a genuinely current review still reaches ready_to_finalize: true

## No-spec Rationale

Not applicable
