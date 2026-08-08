---
id: CHG-0099-ship-status-live-github-check-run-trust-for-product-parent-sha
state: approved
type: feature
base_commit: 2b43f39ff73ed624c54996913b28ac698f60f9c3
---

# Ship-status live GitHub check-run trust for product parent SHA

## Intent

ship-status live GitHub check-run trust for product parent SHA

## Affected Canonical Specs

- `cmd_change`
- `github`

## Acceptance Criteria

- ship-status reports trust.status green|pending|failed|empty|unavailable from GitHub check-runs when GITHUB_TOKEN is set; falls back to local_guidance offline; unit tests cover aggregation

## No-spec Rationale

Not applicable
