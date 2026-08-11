---
change: CHG-0101-add-audited-solo-maintainer-self-review-override
artifact: plan
---

# Plan

1. Extend the `change review` grammar with an explicit `--self-review` branch that requires
   `--actor` and `--reason`; preserve the existing `--reviewer` branch for independent reviews.
2. Add a versioned review-mode/provenance representation that retains v2 independent records and
   records audited self-review identity and reason without claiming a GitHub review check.
3. Make domain validation permit only the approved scope approver in self-review mode, while
   rejecting missing, malformed, mismatched, or ambiguous identity/mode inputs.
4. Project the persisted review mode truthfully through text, JSON, and ship-status guidance.
5. Add parser, domain, and command regression coverage; run the configured scoped checks plus the
   repository trust gate before recording verification.
