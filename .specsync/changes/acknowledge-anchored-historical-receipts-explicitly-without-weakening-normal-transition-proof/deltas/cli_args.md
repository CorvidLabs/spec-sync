## ADDED

### REQUIREMENT REQ-cli-args-020

The CLI SHALL parse `change reconcile-history ID --actor ACTOR --reason REASON` as an explicit historical scope request, requiring both actor and reason, without implying approval or merge.

#### Acceptance Criteria

- Help lists reconcile-history under change.
- Missing actor or reason fails argument parsing. Existing command grammar remains unchanged.
