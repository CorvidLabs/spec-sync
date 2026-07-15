## MODIFIED

### REQUIREMENT REQ-cmd-change-001

The system SHALL provide equivalent text and JSON interfaces for the complete SDD lifecycle and its explicit semantic-succession adoption.

Acceptance Criteria
- Humans receive concise next-action guidance and one consistent accepted-validity reason.
- Agents receive stable records, summaries, question identifiers, manifests, supersedes edges, and successor evidence.
- `change supersede` records explicit predecessor/path/module/digest obligations only before definition approval.
- Audited reopen returns the verifying change and its versioned supersession event in deterministic JSON.
- Active accepted check/status/reopen/archive eligibility render exact, successor-covered, or stale; archived status renders authenticated-history or corrupt-history.


## MODIFIED

### SPEC SECTION Contract

1. Every operation delegates domain policy to the change module.
2. Errors render consistently and exit non-zero.
3. Status and interviews provide a concrete next action and a state-appropriate active-current or archived-history validity reason.
4. Supersede records an explicit digest-bound predecessor/path/module obligation before definition approval.
5. Reopen renders the exact persisted versioned supersession event in deterministic JSON.

### SPEC SECTION Error Cases

| Condition | Behavior |
|-----------|----------|
| Unknown change type | Descriptive error and exit 1 |
| Invalid transition | Current and expected states plus exit 1 |
| Missing actor or reason | Clap or domain validation exits non-zero without lifecycle mutation |
| Current or successor-covered accepted evidence | Reopen reports the shared non-stale reason and exits 1 |
| Missing or mismatched supersede obligation | Command reports the exact predecessor/path/module/digest mismatch and exits 1 without definition mutation |
