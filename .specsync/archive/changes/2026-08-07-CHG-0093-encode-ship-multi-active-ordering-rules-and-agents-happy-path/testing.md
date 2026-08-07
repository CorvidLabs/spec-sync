---
change: CHG-0093-encode-ship-multi-active-ordering-rules-and-agents-happy-path
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|---------|
| REQ-cmd-change-007 | ship-status JSON gains sibling_active_ids + multi-active warnings; unit test multi_active_ordering_warnings_encode_four_rules |
| REQ-cmd-change-008 | ship finalize next guidance names remaining siblings |

## Manual

- cargo test commands::change::tests::
- change ship-status with two actives shows sibling warnings
