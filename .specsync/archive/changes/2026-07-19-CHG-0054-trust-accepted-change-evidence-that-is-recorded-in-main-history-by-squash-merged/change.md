---
id: CHG-0054-trust-accepted-change-evidence-that-is-recorded-in-main-history-by-squash-merged
state: archived
type: bug_fix
base_commit: b5218aac7de501cce60f6cbbaba3b72324427ecf
---

# Trust accepted-change evidence that is recorded in main history by squash-merged commits so accepted and archived changes whose verification and closing approval bytes match an in-history accepted record can be archived even when the original acceptance-transition commit was discarded by a squash merge

## Intent

Trust accepted-change evidence that is recorded in main history by squash-merged commits so accepted and archived changes whose verification and closing approval bytes match an in-history accepted record can be archived even when the original acceptance-transition commit was discarded by a squash merge

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- An accepted change whose acceptance evidence was refreshed while already accepted and squash-merged (no in-history first-acceptance commit carrying current bytes, and a verification commit that is not an ancestor of HEAD) archives successfully when an in-history commit records the change as accepted with byte-identical state, verification, and approvals; changes with no matching in-history accepted record still fail closed with the existing diagnostic; all four currently blocked changes (CHG-0048 musl, CHG-0051, CHG-0052, CHG-0053) archive successfully with the fixed binary; regression tests cover the squash-merged evidence path.

## No-spec Rationale

Not applicable
