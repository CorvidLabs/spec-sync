## MODIFIED

### REQUIREMENT REQ-cmd-change-ship-001

The cmd_change surface SHALL expose `specsync change ship-status` that reports HEAD tip class and the staged product→review→archive ship sequence for workflow-v2 close-out. CI/Trust confidence split is documented so ship guidance does not imply a second test suite.

Acceptance Criteria

- ship-status prints or returns tip_class and ship_sequence.
- Optional change id scopes lifecycle fields when multiple delivering changes exist.
- docs/ci-confidence.md describes CI as multi-OS suite authority and Trust without duplicate cargo test.

### REQUIREMENT REQ-cmd-change-ship-002

When a change is verifying with scoped review evidence present, next_action guidance SHALL prefer the staged ship sequence over a vague finalize-only hint.

Acceptance Criteria

- Text-mode next_action after review mentions product tip green / finalize / archive tip or ship-status.
