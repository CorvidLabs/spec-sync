---
change: CHG-0018-allow-section-only-semantic-deltas-to-satisfy-verification-evidence
artifact: context
---

# Context

Verification currently equates semantic acceptance evidence with collected requirement IDs. A valid delta that only modifies a canonical spec section therefore runs its configured command successfully but persists `passed: false` and reports the command as failed. Spec sections are first-class parsed semantic delta items and need equivalent acceptance-evidence recognition without weakening requirement-to-test mappings.
