---
id: cover-the-two-paths-900-changed-outside-its-declared-scope
state: draft
type: bug_fix
base_commit: ccc35abd6013041c37d2737e2dcb3290996f9b58
---

# Cover the two paths #900 changed outside its declared scope

## Intent

Cover the two paths #900 changed outside its declared scope

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- An approval tells me it matches the content it approved and nothing more; the name on it is a claim, not proof of who gave it.
- The change's status shows it beside the approval or review it belongs to.

## No-spec Rationale

#900 already changed these two files and Leif reviewed them at 1b21d31f: correction_prefix_digest hashes the correction without embedded relays (the #900 P1 fix), and the exact status key-set test expects recorded_claims. This change only declares them; no spec text and no code changes.
