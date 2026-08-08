---
change: CHG-0099-ship-status-live-github-check-run-trust-for-product-parent-sha
artifact: testing
---

# Testing

## REQ-cmd-change-007

- Manual: `SPECSYNC_SHIP_LOCAL_GUIDANCE=1 change ship-status` stays local_guidance.
- With token: `--json change ship-status` shows trust.source=github_check_runs when remote is GitHub.

## REQ-github-010

- `cargo test parse_commit_check_summary_aggregates_green_pending_failed`

Evidence IDs: REQ-cmd-change-007, REQ-github-010
