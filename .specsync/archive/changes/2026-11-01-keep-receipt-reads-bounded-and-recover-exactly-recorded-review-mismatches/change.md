---
id: keep-receipt-reads-bounded-and-recover-exactly-recorded-review-mismatches
state: archived
type: bug_fix
base_commit: 3d3d6e6db88077b1c233b37e4958916d8da40bbd
---

# Keep receipt reads bounded and recover exactly recorded review mismatches

## Intent

Keep receipt reads bounded and recover exactly recorded review mismatches

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Restrict recursive frozen-receipt reads to signed entries while retaining full fresh/reopen discovery and all bounds; admit reopen-only recovery of an exactly remote-default recorded active v2 review-workspace mismatch without trusting it; preserve original bytes through an ancestor Git anchor, require fresh verification and review, reject tampering and other invalid digests without writes, and retain backward-compatible optional audit metadata.

## No-spec Rationale

Not applicable
