---
module: change
---

## ADDED

### REQUIREMENT REQ-change-154

A `change check` refused on grounds in the change's own workspace — open
tasks or missing requirement evidence — SHALL append an attempt with
`passed: false` and the refusal reason to the change's
`verification-attempts.json`, so a reader can see the check was tried and why
it failed. A refusal SHALL NOT write `verification.json`, change the recorded
state, or discard earlier attempts. A gate failure about another change's
module records nothing, as before.

#### Acceptance Criteria

- A tasks-gate refusal leaves one `passed: false` attempt naming the reason.
- `verification.json` is absent (or unchanged) after a refusal.
- The refusal's exit code and stderr text are unchanged.
