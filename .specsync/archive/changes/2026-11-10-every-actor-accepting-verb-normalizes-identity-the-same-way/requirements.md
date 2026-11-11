---
change: every-actor-accepting-verb-normalizes-identity-the-same-way
artifact: requirements
---

# Requirements


- Every `--actor`-accepting lifecycle verb trims padding and rejects names containing newlines or control characters before any ledger write.
- The empty-actor error keeps naming its verb.
