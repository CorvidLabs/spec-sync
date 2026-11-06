---
change: keep-the-upstream-archive-status-guidance-behind-a-sequence-ledger-freeze
artifact: requirements
---

# Requirements

REQ-change-138 (new, change): the status summary's handoff for an upstream-archived change follows the same priority as `next_action`, so a sequence-ledger freeze or verification_ignored refusal comes first.

REQ-cmd-change-034 (new, cmd_change): text status and the ship-status lifecycle next action give a sequence-ledger freeze before the upstream-archive step.
