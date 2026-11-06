## ADDED

### REQUIREMENT REQ-cmd-change-033

The `change ship-status` product-tip stage SHALL advertise only a `specsync change` command whose state guard admits the change's current state: `change approve` for a draft, `change check <id> --commit` for approved, implementing and verifying, and `change reopen <id> --actor <name> --reason <text>` for accepted and archived changes whose verification is no longer current.

Acceptance Criteria
- An accepted or archived change with stale verification is told to reopen, not to check.
- A draft is told to approve, not to check.
- Approved, implementing and verifying changes are still told to check.
