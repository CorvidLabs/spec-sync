---
change: CHG-0049-make-stale-accepted-change-verification-diagnostics-actionable-with-named-delive
artifact: plan
---

# Plan

1. Rewrite the stale-input reason sites in `validate_accepted_inputs_recursive` so each names the
   delivery input path, owner, and remediation (`change reopen`, covering-successor completion,
   or file restore).
2. Collect covering-but-stale successor IDs during successor evaluation and report them in sorted
   order when no successor is closing-valid.
3. Add unit coverage for the uncovered-input, covering-stale-successor, disappeared-input, and
   exact-only-input messages; extend the CLI integration regression for the stale check error.
4. Add spec deltas for `REQ-change-034` and the `change` Error Cases section, then update the
   canonical spec and companions on acceptance.
