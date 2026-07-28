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
- Authenticate every schema-v2 reopening against its exact protected prior accepted root before
  permitting another lifecycle mutation.
- Preserve schema-v1 history only when one protected closing-valid accepted root contains the exact
  complete schema-v1 ledger prefix.
- Resolve the protected remote-default reference only through the target repository's
  environment-sanitized Git view.
- Preserve schema-v1 embedded reopening ledgers without a mandatory rewrite.
- Preserve Git object kind and mode while authenticating legacy acceptance snapshots on every
  supported host platform.
- Keep the public lifecycle behavior, closing authentication, stale-input binding, and archive
  integrity unchanged.
- Bound serialized ledger growth by newly appended event metadata and distinct manifest content.

## Compatibility

- Existing embedded reopening records remain readable and independently verifiable.
- A protected exact-ledger compatibility root may authenticate only the initial contiguous
  schema-v1 prefix; every subsequent schema-v2 event still requires its own prior accepted root.
- New compact records are versioned and reject unsupported or mixed representations.
- `migrate 5.0` continues its reopening-digest repair only and leaves already-valid compact records
  and unrelated legacy records byte-identical.
- No lifecycle command or flag changes.
