---
change: CHG-0099-ship-status-live-github-check-run-trust-for-product-parent-sha
artifact: requirements
---

# Requirements

## REQ-cmd-change-007

Ship-status reports local readiness and optional live GitHub check-run trust.

## REQ-github-010

In-process check-run summary for commit SHAs without spawning gh.

## Acceptance

- unit: parse_commit_check_summary aggregates green/pending/failed/empty
- offline ship-status keeps local_guidance without GITHUB_TOKEN
