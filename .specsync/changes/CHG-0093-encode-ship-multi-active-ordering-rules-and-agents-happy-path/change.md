---
id: CHG-0093-encode-ship-multi-active-ordering-rules-and-agents-happy-path
state: implementing
type: feature
base_commit: b36809a7d673261e1d4b39a55728bce6eb492427
---

# Encode ship multi-active ordering rules and AGENTS happy path

## Intent

encode ship multi-active ordering rules and AGENTS happy path

## Affected Canonical Specs

- `cmd_change`

## Acceptance Criteria

- ship-status and ship warn when other active changes exist; finalize-one-at-a-time and review+ship atomic rules appear in CLI output; Agents.md documents the ship happy path and the four ordering rules

## No-spec Rationale

Surface the four ship ordering rules from #487 dogfood so agents stop batching review/finalize and merging with active changes
