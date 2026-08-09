## ADDED

### REQUIREMENT REQ-change-056

The change domain SHALL expose correction-ledger health to text lifecycle inspection without
returning correction values, ledger bytes, or digest material to a human output path.

Acceptance Criteria

- Malformed, unauthenticated, or otherwise invalid correction history produces a deterministic
  invalid-health result.
- The text-facing diagnostic is generic, names the correction ledger, and directs restoration
  from trusted history.
- The diagnostic contains no correction value, ledger fragment, or digest.
- Valid correction history continues to permit normal text lifecycle inspection.
