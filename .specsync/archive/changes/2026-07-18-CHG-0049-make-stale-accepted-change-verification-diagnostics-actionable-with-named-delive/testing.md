---
change: CHG-0049-make-stale-accepted-change-verification-diagnostics-actionable-with-named-delive
artifact: testing
---

# Testing

Map `REQ-change-034` to focused unit coverage in `src/change.rs` and CLI regression coverage in
`tests/integration/change.rs`. The unit tests drive `check_project` on synthetic workspaces; the
integration test proves the actionable text reaches the `change check` stderr surface.

- Accept a change, modify its covered source input, and assert the stale error names the input
  path, the owner module, and the exact `specsync change reopen <id>` remediation.
- Accept a predecessor, then accept a successor that structurally covers the same input and stale
  them both; assert the predecessor error names the covering successor change ID and its stale
  evidence state instead of the bare reopen remediation.
- Accept a documentation change covering delivery metadata and assert the exact-only
  audited-reopen message names the path and the reopen command at both the unit and CLI surfaces.
- Assert the CLI `change check` failure keeps the established
  `accepted change verification is stale for current delivery inputs` prefix while naming the
  input and remediation, and that repeated runs produce byte-identical messages.
