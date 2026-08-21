---
id: a-reopen-must-extend-the-committed-ledger-not-merely-count-itself
state: implementing
type: bug_fix
base_commit: 7cbe820ebc1da9160f6711dc9e0f7058459a7162
---

# A reopen must extend the committed ledger, not merely count itself

## Intent

a reopen must extend the committed ledger, not merely count itself

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- The #660 anchor fix bounded acceptance anchors to the earliest commit that introduced a package, and used approval_ledger_generation, the length of approvals.json reopenings, to tell a legitimate reopen apart from a forged one. That count is written by whoever writes the file, so it is not evidence: one hand-written ReopenRecord relaunders vectors four, five and six on the current binary, which report authenticated-history where they should report corrupt. The same fix also gated stage D, the working-tree closing evidence fallback, on the package already being in history. Stage D is the only stage that ever supplies an anchor in a reopen lifecycle, because acceptance is reached in the working tree between review and finalize and never committed, so a reopened change could no longer be finalized at all. Done when: a new generation must contain the committed ledger history unrewritten rather than merely outnumber it; the working-tree fallback is available to the process writing a package out of the active workspace and to nothing else; all three laundering vectors are refused with and without a forged generation; a reopened change finalizes again; and no archive that authenticates today stops doing so.

## No-spec Rationale

Not applicable
