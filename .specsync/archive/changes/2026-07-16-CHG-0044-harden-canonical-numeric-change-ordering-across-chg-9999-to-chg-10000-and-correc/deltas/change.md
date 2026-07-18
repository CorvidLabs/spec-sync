## MODIFIED

### REQUIREMENT REQ-change-026

The lifecycle SHALL treat canonical numeric sequence claims and historical collision acknowledgements as protected exact repository evidence across arbitrarily wide numeric sequences.

Acceptance Criteria

- Numeric change sequences contain at least four ASCII digits, use exactly four zero-padded digits below 10000, and use unpadded decimal digits at or above 10000.
- Successor identity ordering compares parsed numeric sequence first and full canonical ID second, so `CHG-10000-*` follows `CHG-9999-*` while acknowledged same-sequence collisions remain deterministic.
- Malformed, noncanonical-width, and numerically unrepresentable IDs fail closed instead of participating in successor ordering.
- The committed sequence ledger always requires lifecycle coverage even when `.specsync/` is ignored.
- Every newly allocated change automatically includes its generated sequence-ledger claim in its affected path scope.
- An acknowledgement matches the exact currently located ID set and remains valid only when every member is accepted or archived.
- Removed IDs, added IDs, single surviving records, and draft, approved, implementing, or verifying collision members fail closed.
