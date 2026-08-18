---
id: CHG-0145-the-sequence-ledger-floor-must-be-wired-not-merely-present
state: archived
type: bug_fix
base_commit: 48999610542d9e04edff64e69cc4fa794d2d61fb
---

# The sequence ledger floor must be wired, not merely present

## Intent

the sequence ledger floor must be wired, not merely present

## Affected Canonical Specs

- `cmd_change`

## Acceptance Criteria

- a test drives git_commit_all against a tree whose working ledger is below its committed mark and asserts the committed ledger did not regress, so removing the floor call from that function fails the suite instead of leaving it green

## No-spec Rationale

floor_sequence_ledger_to_committed had unit tests but nothing asserted git_commit_all calls it, so deleting the call left the whole suite green while every lifecycle commit staged a stale ledger again
