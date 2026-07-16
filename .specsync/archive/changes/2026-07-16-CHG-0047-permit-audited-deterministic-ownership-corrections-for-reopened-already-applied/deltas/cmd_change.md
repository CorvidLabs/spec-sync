## ADDED

### REQUIREMENT REQ-cmd-change-003

The change command adapter SHALL expose audited exact acceptance-owner correction equivalently in
text and deterministic JSON.

Acceptance Criteria

- `change correct-owner` delegates all ownership policy to the change domain module.
- JSON emits the persisted corrected change record, including its append-only owner correction.
- Human output names the exact path, canonical module, actor, and next definition-approval gate.
- Domain rejection exits non-zero without success output or partial lifecycle mutation.

## MODIFIED

### SPEC SECTION Contract

1. Every operation delegates domain policy to the change module.
2. Errors render consistently and exit non-zero.
3. Status and interviews provide a concrete next action and a state-appropriate active-current or archived-history validity reason.
4. Supersede records an explicit digest-bound predecessor/path/module obligation before definition approval.
5. Reopen renders the exact persisted versioned supersession event in deterministic JSON.
6. Correct-owner renders one persisted exact canonical-owner correction and directs the user to definition reapproval.

### SPEC SECTION Error Cases

| Condition | Behavior |
|-----------|----------|
| Unknown change type | Descriptive error and exit 1 |
| Invalid transition | Current and expected states plus exit 1 |
| Missing actor or reason | Clap or domain validation exits non-zero without lifecycle mutation |
| Current or successor-covered accepted evidence | Reopen reports the shared non-stale reason and exits 1 |
| Missing or mismatched supersede obligation | Command reports the exact predecessor/path/module/digest mismatch and exits 1 without definition mutation |
| Invalid exact owner correction | Command reports the domain rejection and exits 1 without lifecycle mutation |
