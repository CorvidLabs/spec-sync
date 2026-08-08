---
change: CHG-0100-ship-push-wait-archive-tip-orchestration-for-buttery-multi-tip-ship
artifact: testing
---

# testing

## Purpose

Complete the #487 multi-tip ship remainder: after finalize, agents can
`change ship --push` the archive tip and `--wait` for GitHub check-runs using the
live trust query from #527.

## Acceptance

- `--push` commits archive package when dirty and runs `git push`
- `--wait` polls check-runs on HEAD until green/failed/timeout
- `--dry-run` rejects combination with `--push`/`--wait`
- No token / SPECSYNC_SHIP_LOCAL_GUIDANCE skips wait with local_guidance status
