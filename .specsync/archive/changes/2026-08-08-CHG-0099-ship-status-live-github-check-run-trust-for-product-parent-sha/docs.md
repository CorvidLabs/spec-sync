---
change: CHG-0099-ship-status-live-github-check-run-trust-for-product-parent-sha
artifact: docs
---

# docs

## Purpose

`change ship-status` must surface **live** GitHub Actions check-run trust for the
product parent SHA when `GITHUB_TOKEN` is available, so agents wait for green CI
before pushing review/archive tips. Offline and no-token stay on local guidance.

## Details

- New REST reader: `fetch_commit_check_summary` (in-process, never spawns `gh`).
- Aggregates check-runs to green / pending / failed / empty.
- `SPECSYNC_SHIP_LOCAL_GUIDANCE=1` forces local guidance for offline CI/sandbox.
- Soft-fail: lookup errors become `status=unavailable` without breaking ship-status.

