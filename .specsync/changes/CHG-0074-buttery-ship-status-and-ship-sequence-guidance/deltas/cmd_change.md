## ADDED

### REQUIREMENT REQ-cmd-change-ship-001

The cmd_change surface SHALL expose `specsync change ship-status` that reports HEAD tip class and the staged product→review→archive ship sequence for workflow-v2 close-out.

Acceptance Criteria

- ship-status prints or returns tip_class and ship_sequence.
- Optional change id scopes lifecycle fields when multiple delivering changes exist.

### REQUIREMENT REQ-cmd-change-ship-002

When a change is verifying with scoped review evidence present, next_action guidance SHALL prefer the staged ship sequence over a vague finalize-only hint.

Acceptance Criteria

- Text-mode next_action after review mentions product tip green / finalize / archive tip or ship-status.
