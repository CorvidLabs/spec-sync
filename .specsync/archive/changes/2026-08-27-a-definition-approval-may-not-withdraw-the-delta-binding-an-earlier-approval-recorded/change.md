---
id: a-definition-approval-may-not-withdraw-the-delta-binding-an-earlier-approval-recorded
state: archived
type: bug_fix
base_commit: 62b297a4eb1822ec444460a172d6264317ebbf2e
---

# A definition approval may not withdraw the delta binding an earlier approval recorded

## Intent

a definition approval may not withdraw the delta binding an earlier approval recorded

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- change approve --portable-5-0-1 records the delta wording it approves, so the effective definition approval of a change that already recorded delta digests never drops to no claim; a ledger whose latest definition approval records no delta digests while an earlier definition approval did is refused at materialization and acceptance with a message naming the re-approve remedy; a ledger in which no definition approval ever recorded a delta digest still materializes unchanged

## No-spec Rationale

Not applicable
