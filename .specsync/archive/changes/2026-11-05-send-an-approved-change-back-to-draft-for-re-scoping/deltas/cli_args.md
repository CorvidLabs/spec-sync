## ADDED

### REQUIREMENT REQ-cli-args-021

The CLI SHALL parse `change send-back ID --actor ACTOR --reason REASON`, requiring both actor and reason.

#### Acceptance Criteria

- Help lists send-back under change.
- Missing actor or reason fails argument parsing. Existing command grammar remains unchanged.
