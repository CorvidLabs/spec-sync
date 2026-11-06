---
change: make-the-ship-status-product-stage-advertise-only-a-command-its-state-accepts
artifact: testing
---

# Testing

`ship_stage_product_action_is_accepted_by_the_state_machine` (`src/commands/change.rs` tests) builds a record, sets each of the six states, calls `ship_stages` with verification present but not current, reads the `product_tip` action, extracts the `specsync change <verb>`, and asserts the verb's guard admits the state (check: approved/implementing/verifying; reopen: accepted/archived; approve/answer: draft). On main cd8cab5e it fails: "Draft: product stage advertises `check`, which refuses that state".

The existing ship-status tests stay green.
