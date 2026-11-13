---
id: a-malformed-relay-in-a-correction-gets-the-repair-relays-recovery
state: draft
type: bug_fix
base_commit: 5d89b2b22a363eda96c8684ba513b07ab2d0a206
---

# A malformed relay in a correction gets the repair-relays recovery

## Intent

A malformed relay in a correction gets the repair-relays recovery

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- A malformed relayed_by in an approval a corrections.json correction embeds, or one naming someone other than that approval's actor, is refused naming specsync change repair-relays <id> for an active change, and git checkout HEAD -- <file> for an archived one; repair-relays then fixes it and the ledger reads (red then green)

## No-spec Rationale

Makes the correction ledger's read match REQ-change-relayed-decision-001 and REQ-change-relay-repair-001: a damaged relay is refused with its recovery, repair-relays for an active change and git checkout HEAD for an archived one. load_correction_ledger reported a bare invalid correction ledger.
