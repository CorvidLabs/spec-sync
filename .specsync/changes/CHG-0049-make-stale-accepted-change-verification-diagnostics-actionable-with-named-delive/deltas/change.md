## ADDED

### REQUIREMENT REQ-change-034

Stale accepted-change verification diagnostics SHALL name the offending delivery input and state
the concrete remediation, without changing the underlying freshness model.

Acceptance Criteria

- A changed covered input with no covering accepted or archived successor reports the input path,
  its owner module, and the `specsync change reopen <id>` remediation.
- A changed covered input whose only covering successors carry stale evidence of their own reports
  the input path, its owner module, and the sorted covering successor change IDs, and directs the
  operator to verify and accept a covering successor or reopen the accepted change.
- A covered input that disappeared from the current inventory reports the missing path and the
  restore-or-reopen remediation; a changed exact-only input reports the path and the audited-reopen
  remediation; missing delivery-input evidence keeps its established phrase and gains the reopen
  remediation.
- Every stale reason remains deterministic: sorted successor IDs, no timestamps, and no
  environment-dependent content.
- The `accepted change verification is stale for current delivery inputs` check prefix, the
  terminal-evidence validity values, and every freshness predicate remain unchanged.

## MODIFIED

### SPEC SECTION Error Cases

| Condition | Behavior |
|-----------|----------|
| Missing acceptance criteria or affected scope | Definition approval fails |
| Missing or invalid semantic delta | Approval, verification, and unified check fail |
| Verification command contains shell operators | Command is rejected without execution |
| HEAD changes after verification | Acceptance requires re-verification |
| Any intervening commit changes a disallowed path, even if later reverted | Status and strict checking require re-verification in every environment |
| Accepted delivery evidence is still current | Reopen is rejected without changing lifecycle or audit state |
| Reopen actor or reason is empty | Reopen is rejected before any mutation |
| Concurrent changes edit the same semantic key | Progress requires dependency ordering or rebase |
| Ownership correction is not exact, additive, in-scope, and canonically provable | Correction is rejected transactionally |
| Covered delivery input of an accepted change changes with no covering accepted successor | Unified check names the input path, its owner, and the `change reopen` remediation |
| Covered delivery input changes while every covering successor is itself stale | Unified check names the input, the sorted covering successor IDs, and their stale evidence state |
| Covered delivery input disappears from the current inventory | Unified check names the missing path and the restore-or-reopen remediation |
