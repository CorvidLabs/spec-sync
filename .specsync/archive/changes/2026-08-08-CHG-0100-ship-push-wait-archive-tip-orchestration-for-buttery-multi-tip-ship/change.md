---
id: CHG-0100-ship-push-wait-archive-tip-orchestration-for-buttery-multi-tip-ship
state: archived
type: feature
base_commit: c6e76b34efd111b40a22ab16b6bc45be692dbe22
---

# Ship --push --wait archive tip orchestration for buttery multi-tip ship

## Intent

ship --push --wait archive tip orchestration for buttery multi-tip ship

## Affected Canonical Specs

- `cmd_change`
- `cli_args`

## Acceptance Criteria

- ship --push commits archive tip and git push; ship --wait polls check-runs until green/failed/timeout; dry-run cannot combine with push/wait; offline no-token wait returns local_guidance

## No-spec Rationale

Not applicable
