---
change: CHG-0049-make-stale-accepted-change-verification-diagnostics-actionable-with-named-delive
artifact: requirements
---

# Requirements

### REQ-change-034

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
