---
change: CHG-0093-encode-ship-multi-active-ordering-rules-and-agents-happy-path
artifact: requirements
---

# Requirements

## Intent

Surface the four ship ordering rules from #487 dogfood in CLI warnings and agent docs.

## Acceptance

- ship-status warns when other active changes exist (finalize one-at-a-time, no batch reviews, no merge with actives)
- ship after finalize points at remaining siblings
- Agents.md has ship happy path + four rules
- REQ-cmd-change-007/008 updated; cmd_change spec version 17
