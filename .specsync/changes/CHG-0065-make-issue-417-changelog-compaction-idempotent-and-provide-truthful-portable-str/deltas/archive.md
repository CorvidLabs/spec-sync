## MODIFIED

### REQUIREMENT REQ-archive-001

The archive module SHALL move completed tasks without losing active content and SHALL report every
selected, succeeded, failed, and rolled-back filesystem operation.

Acceptance Criteria

- Dry-run returns the complete plan without writing.
- Apply mode reads and preflights every target, stages same-directory replacements, and publishes
  atomically only after staging succeeds everywhere.
- Any preflight/staging failure leaves every destination byte-for-byte unchanged.
- Late publish or rollback failures are explicit incomplete/partial outcomes.
- Original destination permissions are preserved.
- Structured failures contain the exact path, operation phase, and bounded error.

### SPEC SECTION Invariants

8. Maintenance application uses plan, preflight, stage, publish, and rollback phases.
9. No preflight/staging failure mutates a destination.
10. The report never claims complete success after a failed operation.
