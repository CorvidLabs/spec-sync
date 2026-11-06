---
id: refuse-review-status-and-finalize-of-a-change-already-archived-on-the-remote-default-and-name-it
state: implementing
type: bug_fix
base_commit: 884a33d3c0cb95680dae786ff25688569ee16c96
---

# Refuse review status and finalize of a change already archived on the remote default and name it

## Intent

Refuse review status and finalize of a change already archived on the remote default and name it

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- When the remote default branch holds an archive of the same change id and `HEAD` does not contain that archive, `change review` refuses before recording acceptance. The refusal names the change, the reference and the commit that archived it, and gives the next step: merge or rebase onto the remote default, whose archive is authoritative. The change is not moved to `accepted`.
- An archive that `HEAD` already contains is not refused: that is the reopen-and-rearchive path, which keeps working unchanged.
- For a refused change, `change status` and `change ship-status` report it as already archived upstream with the same reference, commit and next step. The handoff is never `safe` and the next action is never `finalize` or `ship`.
- `change finalize` on that change refuses with the same named cause instead of "requires exactly one trusted transition ... found 0". It still refuses: no second archive is written.
- Without an upstream archive, a change verified on a branch and squash-merged onto its base before finalize still passes `change review` and `change finalize` archives it (regression guard for #690).
- Both cases are integration tests built from the auditor's squash steps. Every other gate, including finalize's trusted-transition preflight, is unchanged.

## No-spec Rationale

Not applicable
