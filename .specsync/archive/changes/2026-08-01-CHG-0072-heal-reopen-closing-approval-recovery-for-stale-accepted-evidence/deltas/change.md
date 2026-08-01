## MODIFIED

### REQUIREMENT REQ-change-034

The change lifecycle SHALL allow accepted evidence to be reopened when delivery inputs are stale, even if verification.json tip no longer matches the closing approval, by binding reopen to the historical verification attempt that authenticates the closing digest.

Acceptance Criteria
- Reopen succeeds when attempt history contains the acceptance-bound verification the closing approval signed.
- After reopen, re-verify and re-accept (or finalize on workflow v2) restore a matching closing approval.
- Definition approval can be refreshed while accepted when the definition digest is stale.
