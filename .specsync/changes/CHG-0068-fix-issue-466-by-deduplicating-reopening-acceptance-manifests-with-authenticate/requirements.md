---
change: CHG-0068-fix-issue-466-by-deduplicating-reopening-acceptance-manifests-with-authenticate
artifact: requirements
---

# Requirements

## Problem

Every audited reopen currently copies the prior verification's complete acceptance manifest into
`approvals.json`. For broad scopes, repeated stale-evidence recovery therefore grows approximately
with reopening count multiplied by manifest size even when the same manifest was already recorded.

## Required Outcomes

- Persist each distinct prior acceptance manifest once as an immutable content-addressed object.
- Store only a bounded authenticated reference in each new reopening event.
- Hydrate and validate every reference before treating reopening history as trusted.
- Preserve schema-v1 embedded reopening ledgers without a mandatory rewrite.
- Keep the public lifecycle behavior, closing authentication, stale-input binding, and archive
  integrity unchanged.
- Bound serialized ledger growth by newly appended event metadata and distinct manifest content.

## Compatibility

- Existing embedded reopening records remain readable and independently verifiable.
- New compact records are versioned and reject unsupported or mixed representations.
- `migrate 5.0` continues its reopening-digest repair only and leaves already-valid compact records
  and unrelated legacy records byte-identical.
- No lifecycle command or flag changes.
