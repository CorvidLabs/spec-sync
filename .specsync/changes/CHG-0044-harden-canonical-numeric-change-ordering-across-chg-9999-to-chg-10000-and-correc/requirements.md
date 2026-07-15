---
change: CHG-0044-harden-canonical-numeric-change-ordering-across-chg-9999-to-chg-10000-and-correc
artifact: requirements
---

# Requirements

## REQ-CHG-0044-001 — Canonical numeric change ordering

SpecSync SHALL order change identities by their canonical numeric sequence rather than whole-ID lexicographic order.

Acceptance criteria:

- `CHG-10000-*` sorts after `CHG-9999-*`.
- Sequences below 10000 use exactly four zero-padded digits; wider sequences use unpadded decimal digits.
- Same-sequence acknowledged collisions use the full canonical ID as a deterministic secondary key.
- If either identity is malformed, noncanonical, or numerically unrepresentable, successor ordering fails closed.
- Existing canonical four-digit and wider IDs retain their numeric identity.

## REQ-CHG-0044-002 — Accurate 5.1 release description

The unreleased 5.1 materials SHALL describe accepted behavior and current release state accurately.

Acceptance criteria:

- The changelog retains `.mjs` and `.cjs` discovery and strict coverage behavior.
- The changelog states that extensionless export-star targets resolve sibling `.mjs` and `.cjs` modules.
- Both adversarial-proof comparison tables identify SpecSync 5.1.
- The Trust workflow identifies the pinned SHA as an immutable unreleased candidate and does not claim a v1.0.1 tag.
- SpecSync 5.1.0 precedes Trust 1.0.1 in the release sequence; CHG44 creates neither release.
