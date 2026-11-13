---
id: text-change-status-names-a-sequence-ledger-freeze-before-the-verification-ignored-paths-repair
state: approved
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# Text change status names a sequence-ledger freeze before the verification_ignored_paths repair

## Intent

Text change status names a sequence-ledger freeze before the verification_ignored_paths repair

## Affected Canonical Specs

- `cmd_change`

## Acceptance Criteria

- with a frozen sequence ledger and an invalid verification_ignored_paths entry, text change status Next names the freeze, as JSON next_action does; without a freeze, text Next still names the verification_ignored_paths repair

## No-spec Rationale

REQ-cmd-change-034 requires text change status to name its next action matching JSON status, and REQ-change-138 orders JSON next_action as a sequence-ledger freeze, then a verification_ignored_paths refusal; text named the policy repair first (#937)
