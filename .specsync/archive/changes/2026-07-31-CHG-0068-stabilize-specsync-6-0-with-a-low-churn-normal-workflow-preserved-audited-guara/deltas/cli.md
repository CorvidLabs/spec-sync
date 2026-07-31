## ADDED

### REQUIREMENT REQ-cli-008

The root command dispatcher SHALL carry strict verification into the same discoverable change
workflow without adding a lifecycle profile or an external merge action.

Acceptance Criteria

- Global `--strict` adds validators to `change check` evidence and does not change lifecycle states,
  approvals, artifact layout, review, finalization, or archive behavior.
- `change status` and structured output expose the same single next action.
- `change finalize` prepares the PR for GitHub merge and never invokes a provider merge API.
- Historical repair commands remain dispatchable for existing evidence without appearing in the
  newcomer core path.
