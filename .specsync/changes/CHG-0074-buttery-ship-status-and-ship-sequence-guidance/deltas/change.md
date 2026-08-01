## MODIFIED

### REQUIREMENT REQ-change-047

The change lifecycle SHALL prefer completing incomplete selected artifacts over definition approval for draft changes once the interview is complete and artifact completeness validation fails. Ship close-out guidance is additionally surfaced via change ship-status without weakening this rule.

Acceptance Criteria

- When selected artifacts contain incomplete HTML TODO comment stubs or are empty, summarize_change sets artifacts_complete to false and next_action does not recommend change approve.
- After selected artifacts are complete, draft next action may recommend definition approval.
- Verifying changes may point agents at ship-status for staged tip close-out without recommending approve while incomplete.
