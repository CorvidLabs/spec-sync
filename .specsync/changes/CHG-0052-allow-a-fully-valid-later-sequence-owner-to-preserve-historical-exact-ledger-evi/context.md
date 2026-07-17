---
change: CHG-0052-allow-a-fully-valid-later-sequence-owner-to-preserve-historical-exact-ledger-evi
artifact: context
---

# Context

The main branch and the 5.1.1 release branch each accepted a distinct CHG-0048 before
integration. The merged ledger must acknowledge both immutable records. That acknowledgement was
not present when CHG-0048, CHG-0049, and CHG-0050 recorded their exact delivery evidence, so a
forced strict check correctly exposes different historical ledger bytes.

REQ-change-029 and invariant 14 already require a fully valid later sequence claim to govern only
this historical ledger transition. The validator currently rejects every changed `@exact` entry
before applying that special sequence rule, leaving the documented behavior unimplemented.
