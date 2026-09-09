## MODIFIED

### REQUIREMENT REQ-change-100

A workflow-v1 change in `Verifying` SHALL advertise `specsync change verify` then `specsync change accept` then `specsync change archive` as `next_action`, never the workflow-v2 `check` / `review` / `finalize` verbs.

Acceptance Criteria
- `summarize_change` for `workflow_version < 2` and `ChangeState::Verifying` (artifacts complete, approval valid) names `verify`, `accept`, and `archive` in that order.
- The `Accepted` arm is unchanged: v2 still names `finalize`; v1 still names `archive` and does not skip `accept`.
- A workflow-v2 Verifying change still names `check` / `review` / `finalize`.
- Tests that assert this contract SHALL NOT interpolate the full `next_action` string into panic messages.
