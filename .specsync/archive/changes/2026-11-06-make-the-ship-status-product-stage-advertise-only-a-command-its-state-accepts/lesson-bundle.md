# Lesson bundle: make-the-ship-status-product-stage-advertise-only-a-command-its-state-accepts

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Make the ship-status product stage advertise only a command its state accepts
- **Kind**: BugFix
- **Specs**: cmd_change
- **Paths**: src/commands/change.rs, CHANGELOG.md
- **Acceptance**: For an `accepted` or `archived` change whose verification is no longer current for this content, the `ship-status` product-tip stage action names `specsync change reopen <id> --actor <name> --reason <text>` instead of `change check --commit`, which refuses those states.
- **Acceptance**: For a `draft` change the stage action names `specsync change approve <id> --actor <name>` instead of `change check --commit`; approved, implementing and verifying changes keep `change check <id> --commit`.
- **Acceptance**: A unit test drives every state through the product stage with stale verification and asserts the advertised `specsync change` verb is one whose state guard admits that state. It fails on main first (draft and accepted advertise `check`).
- **Acceptance**: The top-level `ship_next`, readiness, stage ids and every gate are unchanged.

## Evidence

- Verification commit: `2f9c6528e0633747e06a5631aa31f059dadef1a8`
- Base commit: `cd8cab5e1789246f6f3a4bf6733b33d301d3ca4b`
- Verified by: `specsync check --spec cmd_change`

## From the change's context.md

# Context

spec-sync#690: `ship-status` advertised `specsync change check <id> --commit` for an `accepted` change whose verification was no longer current, and `check` refuses `accepted` ("expected approved or implementing or verifying").

On main cd8cab5e the top-level `ship_next` is already safe: #534's narrowing falls back to the lifecycle next step for draft, accepted and archived. The `product_tip` stage in `ship_stages` (`src/commands/change.rs`) still prints `check --commit` whenever the stage is not done, whatever the state. A unit test over every state shows draft, accepted and archived get a command their guard refuses.

Leif chose the fix 2026-11-05 22:35 ("Take it"): name `change reopen` for accepted. Archived has the same guard (reopen admits accepted and archived), and draft needs `change approve`, so the definition covers all three so the test can hold for every state; this is flagged in the approval request.

## From the change's testing.md

# Testing

`ship_stage_product_action_is_accepted_by_the_state_machine` (`src/commands/change.rs` tests) builds a record, sets each of the six states, calls `ship_stages` with verification present but not current, reads the `product_tip` action, extracts the `specsync change <verb>`, and asserts the verb's guard admits the state (check: approved/implementing/verifying; reopen: accepted/archived; approve/answer: draft). On main cd8cab5e it fails: "Draft: product stage advertises `check`, which refuses that state".

The existing ship-status tests stay green.

## Results

`ship_stage_product_action_is_accepted_by_the_state_machine` failed on main cd8cab5e at Draft and passes with `product_stage_action`. The other ship-status and ship-stage tests (9 with this one) pass.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-cmd-change-033 | `ship_stage_product_action_is_accepted_by_the_state_machine` |

## Where these lessons go

- `specs/cmd_change/cmd_change.3md`
